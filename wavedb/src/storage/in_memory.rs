use crate::errors::Waverr;
use crate::puddle::{Droplet, Puddle, SignalId, Toffset};
use std::sync::Arc;

#[derive(Debug, Default)]
pub struct InMemWave {
    name: String,
    pub signal_id: SignalId,
    width: u32,
    puddles: Vec<Arc<Puddle>>,
}

impl AsRef<str> for InMemWave {
    fn as_ref(&self) -> &str {
        self.name.as_str()
    }
}

///In memory DS for wave content; created from a Vector of Arcs to puddles
impl InMemWave {
    pub fn all_data(&self) -> impl Iterator<Item = (u32, &[u8])> + '_ {
        let sigid = self.signal_id;
        self.puddles
            .iter()
            .filter_map(move |puddle| {
                puddle
                    .get_cursor(sigid)
                    .ok()
                    .map(|cursor| (cursor, puddle.puddle_base()))
            })
            .flat_map(|(cursor, base)| cursor.into_iter().zip(std::iter::repeat(base)))
            .map(|(droplet, base)| {
                (
                    base + droplet.get_timestamp() as Toffset,
                    droplet.take_data(),
                )
            })
    }

    //FIXME: is there a way to .. minimize the boxing going on here?
    pub fn data_in_range(
        &self,
        begin: Toffset,
        end: Toffset,
    ) -> impl Iterator<Item = (Toffset, &[u8])> + '_ {
        self.droplets_in_range(begin, end)
            .map(|(base, droplet)| (base, droplet.take_data()))
    }

    fn get_idx(&self, time: Toffset) -> Option<usize> {
        self.puddles
            .iter()
            .position(|puddle| puddle.puddle_base() == time & !(Puddle::max_puddle_length() - 1))
    }

    pub fn get_prev_droplet(&self, time: Toffset) -> Option<Droplet<'_>> {
        let idx = self.get_idx(time)?;
        let sigid = self.signal_id;
        self.puddles[0..idx + 1]
            .iter()
            .rev()
            .filter_map(move |puddle| {
                puddle
                    .get_cursor(sigid)
                    .ok()
                    .map(|cursor| (cursor, puddle.puddle_base()))
            })
            .flat_map(|(cursor, base)| cursor.into_iter().rev().zip(std::iter::repeat(base)))
            .filter(|(droplet, base)| ((base + droplet.get_timestamp() as Toffset) < time))
            .map(|(droplet, _base)| droplet)
            .next()
    }


    pub fn get_droplet_at(&self, time: Toffset) -> Option<Droplet<'_>>{
        self.get_prev_droplet(time+1)
    }

    pub fn get_prev_time(&self, time: Toffset) -> Option<(Toffset, &'_ [u8])> {
        let idx = self.get_idx(time)?;
        let sigid = self.signal_id;
        self.puddles[0..idx + 1]
            .iter()
            .rev()
            .filter_map(move |puddle| {
                puddle
                    .get_cursor(sigid)
                    .ok()
                    .map(|cursor| (cursor, puddle.puddle_base()))
            })
            .flat_map(|(cursor, base)| cursor.into_iter().rev().zip(std::iter::repeat(base)))
            .map(|(droplet, base)| {
                (
                    base + droplet.get_timestamp() as Toffset,
                    droplet.take_data(),
                )
            })
            .filter(|(droplet_timestamp, _)| *droplet_timestamp < time)
            .next()
    }

    pub fn get_next_time(&self, time: Toffset) -> Option<(Toffset, &'_ [u8])> {
        let idx = self.get_idx(time)?;
        let sigid = self.signal_id;
        self.puddles[idx..]
            .iter()
            .filter_map(move |puddle| {
                puddle
                    .get_cursor(sigid)
                    .ok()
                    .map(|cursor| (cursor, puddle.puddle_base()))
            })
            .flat_map(|(cursor, base)| cursor.into_iter().zip(std::iter::repeat(base)))
            .map(|(droplet, base)| {
                (
                    base + droplet.get_timestamp() as Toffset,
                    droplet.take_data(),
                )
            })
            .filter(|(droplet_timestamp, _)| *droplet_timestamp > time)
            .next()
    }

    //fixme; could probably template and
    pub fn droplets_in_range(
        &self,
        begin: Toffset,
        end: Toffset,
    ) -> impl Iterator<Item = (Toffset, Droplet<'_>)> + '_ {
        let sigid = self.signal_id;
        self.puddles
            .iter()
            .filter(move |puddle| begin < puddle.puddle_end() && end > puddle.puddle_base())
            .filter_map(move |puddle| {
                puddle
                    .get_cursor(sigid)
                    .ok()
                    .map(|cursor| (cursor, puddle.puddle_base()))
            })
            .flat_map(|(cursor, base)| cursor.into_iter().zip(std::iter::repeat(base)))
            .map(|(droplet, base)| (base + droplet.get_timestamp() as Toffset, droplet))
            .filter(move |(time, _)| *time >= begin && *time < end)
    }

    pub fn get_width(&self) -> usize {
        self.width as usize
    }

    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    pub fn new(
        name_str: String,
        signal_id: SignalId,
        width: u32,
        puddles: Vec<Arc<Puddle>>,
    ) -> Result<InMemWave, Waverr> {
        Ok(InMemWave {
            name: name_str,
            width,
            signal_id,
            puddles,
        })
    }
}

impl std::fmt::Display for InMemWave {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_name())?;
        let width = self.get_width();
        if width > 1 {
            write!(f, " [{}:0]", width - 1)?;
        }
        Ok(())
    }
}

#[cfg(test)]
#[allow(dead_code, unused_macros, unused_imports, unused_variables)]
mod tests {
    use super::*;
    use crate::puddle::builder::tests::build_dummy_puddles;
    use crate::puddle::Droplet;
    use crate::wavedb::WaveDb;
    use std::convert::TryInto;
    use std::path::{Path, PathBuf};

    fn init_test_logger() {
        let _ = env_logger::builder()
            .filter_level(log::LevelFilter::Info)
            .is_test(true)
            .try_init();
    }

    /// Utility to create vga wavedb object
    fn create_vga_wdb() -> WaveDb {
        let mut path_to_wikivcd = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path_to_wikivcd.push("test_vcds/vga.vcd");
        let db = tempfile::TempDir::new().expect("Temp file could not be created! Shucks");

        WaveDb::from_vcd(path_to_wikivcd, db.path()).expect("could not create wavedb")
    }

    #[test]
    fn sanity_imw() {
        let signal_width = 16;
        let puddles: Vec<Arc<Puddle>> = (0..5)
            .into_iter()
            .map(|idx| build_dummy_puddles(idx * Puddle::max_puddle_length(), 20, signal_width))
            .collect();
        let imw_0 = InMemWave::new("sig_0".into(), 0, signal_width as u32, puddles).unwrap();
        let first_puddle_fragment: Vec<(u32, &[u8])> = imw_0.data_in_range(0, 1000).collect();

        assert_eq!(first_puddle_fragment.len(), 1000);
        for (time, payload) in first_puddle_fragment {
            let value = u16::from_le_bytes(payload.try_into().expect("should be u16"));
            assert_eq!(time as u16, value);
        }
    }

    #[test]
    fn vga_clock_in_range() {
        let wdb = create_vga_wdb();
        let clock_wave = wdb.get_imw("TOP.clock".into()).expect("signal isn't here!");
        let mut last_time = 0;
        for (time, payload) in clock_wave.data_in_range(0, 40000) {
            assert!(payload.len() == 1);
            assert!(last_time <= time);
            last_time = time;
        }
    }

    #[test]
    fn vga_x_addr_data_in_range() {
        let wdb = create_vga_wdb();

        let clock_wave = wdb
            .get_imw("TOP.x_addr".into())
            .expect("signal isn't here!");
        let mut expected_val = 0;
        for (time, payload) in clock_wave.data_in_range(0, 10000) {
            log::info!("payload is {:?}", payload);
            log::info!("time is {:?}", time);
            let val: u16 = u16::from_le_bytes(
                payload
                    .try_into()
                    .expect("should be a 9bit val, convertible into u16"),
            );
            if val == 0 {
                expected_val = 0;
            }
            assert_eq!(expected_val, val);
            expected_val += 1;
        }
    }

    #[test]
    fn vga_x_addr_get_next_and_prev_time() {
        let wdb = create_vga_wdb();

        let clock_wave = wdb
            .get_imw("TOP.x_addr".into())
            .expect("signal isn't here!");
        let (toffset, payload) = clock_wave.get_prev_time(16029).expect("prev failed");
        assert_eq!(toffset, 16010);
        let val: u16 = u16::from_le_bytes(
            payload
                .try_into()
                .expect("should be a 9bit val, convertible into u16"),
        );
        assert_eq!(val, 0x280);
        let (toffset, payload) = clock_wave.get_next_time(16030).expect("next time failed");
        let val: u16 = u16::from_le_bytes(
            payload
                .try_into()
                .expect("should be a 9bit val, convertible into u16"),
        );

        assert_eq!(toffset, 19250);
        assert_eq!(val, 0x1);
    }

    #[test]
    fn vga_x_cnt_entire_iterator() {
        let wdb = create_vga_wdb();

        let x_cnt = wdb
            .get_imw("TOP.vga.x_cnt".into())
            .expect("signal isn't here!");
        let mut prev_time = 0;
        for (time, _) in x_cnt.droplets_in_range(6900, 7500) {
            assert!(time > prev_time);
            prev_time = time
        }
    }
}
