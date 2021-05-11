use crate::puddle::Droplet;

#[derive(Clone, Copy, Debug)]
///Represents ways to format ParsedVec into String
pub enum WaveFormat {
    Decimal,
    Hex,
    Octal,
    SDecimal,
}

impl WaveFormat {
    ///The number of bits per digit for this particular radix
    fn num_bits(&self) -> f32 {
        match self {
            WaveFormat::Hex => 4.0,
            WaveFormat::Octal => 3.0,
            WaveFormat::Decimal | WaveFormat::SDecimal => 3.32,
        }
    }
}

fn split_zx_and_payload<'a>(drop: Droplet<'a>, bitwidth: usize) -> (&'a [u8], &'a [u8]) {
    let bitwidth_idx = (bitwidth as f32 / 8.0).ceil() as usize;
    drop.take_data().split_at(bitwidth_idx)
}

pub fn format_payload<'a>(
    drop: Droplet<'a>,
    format: WaveFormat,
    bitwidth: usize,
    visibile_chars: usize,
) -> String {
    match format {
        WaveFormat::Hex => {
            if drop.is_zx() {
                let (payload, zx) = split_zx_and_payload(drop, bitwidth);
                payload
                    .iter()
                    .zip(zx.iter())
                    .rev()
                    .enumerate()
                    .take_while(|(num_chars, _)| *num_chars < visibile_chars / 2)
                    .map(|(_, (chunk, zx))| match zx {
                        0 => {
                            format!("{:x?}", chunk)
                        }
                        0x01..=0x0f => {
                            let zorx = if (zx & chunk) != 0 { "x" } else { "z" };
                            format!("{:x?}{}", chunk >> 4, zorx)
                        }
                        0x10..=0xf0 => {
                            let zorx = if (zx & chunk) != 0 { "x" } else { "z" };
                            format!("{:x?}{}", chunk & 0xf, zorx)

                        }
                        _ => {
                            let zorx1 = if (zx & chunk & 0xf) != 0 { "x" } else { "z" };
                            let zorx2 = if (zx & chunk & 0xf0) != 0 { "x" } else { "z"};
                            format!("{}{}",zorx1,zorx2)
                        }
                    })
                    .collect()
            } else {
                drop.take_data()
                    .iter()
                    .rev()
                    .map(|chunk| format!("{:x?}", chunk))
                    .collect()
            }
        }
        _ => {
            unimplemented!()
        }
    }
}


#[cfg(test)]
#[allow(dead_code, unused_macros, unused_imports, unused_variables)]
mod tests {
    use super::{format_payload, WaveFormat};
    use crate::puddle::testing_utils::test_droplet;


    #[test]
    fn vanilla_hex() {
        let content : Vec<u8> = vec![0,0,0xef,0xbe,0xad,0xde];
        let droplet = test_droplet(content.as_slice());
        let output = format_payload(droplet, WaveFormat::Hex, 32,20);
        assert_eq!(output, String::from("deadbeef"));
    }

}
