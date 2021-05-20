use crate::errors::Waverr;
use crate::puddle::{Droplet, Puddle, SignalId, Toffset};
use std::sync::Arc;

#[derive(Debug, Default)]
pub struct InMemWave {
    name: String,
    signal_id: SignalId,
    puddles: Vec<Arc<Puddle>>,
    //width: Option<u32>,
    ///// The index into the puddles variable for deciding which puddle we should convert into a
    ///// cursor
    //ref_holder: std::marker::PhantomData<&'a u8>
    //live_cursor: Option<PCursor<'a>>
}

///In memory DS for wave content; created from a Vector of Arcs to puddles
impl InMemWave {
    pub fn all_data(&self) -> Box<dyn Iterator<Item = (u32, &[u8])> + '_> {
        let sigid = self.signal_id;
        Box::new(
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
                }),
        )
    }


    //FIXME: is there a way to like.. minimize the boxing going on here?
    pub fn data_in_range(
        &self,
        begin: Toffset,
        end: Toffset,
    ) -> Box<dyn Iterator<Item = (Toffset, &[u8])> + '_> {
        Box::new(
            self.droplets_in_range(begin, end)
                .map(|(base, droplet)| (base, droplet.take_data())),
        )
    }

    //fixme; could probably template and
    pub fn droplets_in_range(
        &self,
        begin: Toffset,
        end: Toffset,
    ) -> Box<dyn Iterator<Item = (Toffset, Droplet<'_>)> + '_> {
        let sigid = self.signal_id;
        Box::new(
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
                .filter(move |(time, _)| *time >= begin && *time < end),
        )
    }

    pub fn get_width(&self) -> usize {
        self.puddles
            .iter()
            .find_map(|puddle| puddle.get_signal_width(self.signal_id))
            .expect("NO WIDTH FUCK")
    }

    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    pub fn new(
        name_str: String,
        signal_id: SignalId,
        puddles: Vec<Arc<Puddle>>,
    ) -> Result<InMemWave, Waverr> {
        Ok(InMemWave {
            name: name_str,
            signal_id,
            puddles,
        })
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

    #[test]
    fn sanity_imw() {
        let puddles: Vec<Arc<Puddle>> = (0..5)
            .into_iter()
            .map(|idx| build_dummy_puddles(idx * Puddle::max_puddle_width(), 20, 16))
            .collect();
        let imw_0 = InMemWave::new("sig_0".into(), 0, puddles).unwrap();
        let first_puddle_fragment: Vec<(u32, &[u8])> = imw_0.data_in_range(0, 1000).collect();

        assert_eq!(first_puddle_fragment.len(), 1000);
        for (time, payload) in first_puddle_fragment {
            let value = u16::from_le_bytes(payload.try_into().expect("should be u16"));
            assert_eq!(time as u16, value);
        }
    }

    #[test]
    fn vga_clock_in_range() {
        let mut path_to_wikivcd = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path_to_wikivcd.push("test_vcds/vga.vcd");
        let db = tempfile::TempDir::new().expect("Temp file could not be created! Shucks");

        let wdb = WaveDb::from_vcd(path_to_wikivcd, db.path()).expect("could not create wavedb");

        let clock_wave = wdb.get_imw("TOP.clock".into()).expect("signal isn't here!");
        let mut last_time = 0;
        for (time, payload) in clock_wave.data_in_range(0, 40000) {
            assert!(payload.len() == 1);
            assert!(last_time <= time);
            last_time = time;
        }
    }

    #[test]
    fn vga_x_addr() {
        let mut path_to_wikivcd = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path_to_wikivcd.push("test_vcds/vga.vcd");
        //bad but hey... is what it is
        let db = tempfile::TempDir::new().expect("Temp file could not be created! Shucks");

        let wdb = WaveDb::from_vcd(path_to_wikivcd, db.path()).expect("could not create wavedb");

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
}
