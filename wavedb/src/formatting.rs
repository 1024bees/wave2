use crate::puddle::utils;
use crate::puddle::Droplet;
#[derive(Clone, Copy, Debug)]
///Represents ways to format ParsedVec into String
pub enum WaveFormat {
    Decimal,
    Hex,
    Binary,
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
            WaveFormat::Binary => 1.0,
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
    visible_chars: usize,
) -> String {
    let visible_chars = visible_chars.min(bitwidth);
    let mut gen_str: String = match format {
        WaveFormat::Hex => {
            if drop.is_zx() {
                let (payload, zx) = split_zx_and_payload(drop, bitwidth);
                payload
                    .iter()
                    .zip(zx.iter())
                    .rev()
                    .enumerate()
                    .take_while(|(num_chars, _)| *num_chars <= visible_chars / 2)
                    .map(|(_, (chunk, zx))| match zx {
                        0 => {
                            format!("{:x?}", chunk)
                        }
                        0x01..=0x0f => {
                            let zorx = if (zx & chunk) != 0 { "x" } else { "z" };
                            format!("{:x?}{}", chunk >> 4, zorx)
                        }
                        i if (i & 0xf0) != 0 && (i & 0x0f) == 0 => {
                            let zorx = if (zx & chunk) != 0 { "x" } else { "z" };
                            format!("{:x?}{}", chunk & 0xf, zorx)
                        }
                        _ => {
                            let zorx1 = if (zx & chunk & 0xf) != 0 { "x" } else { "z" };
                            let zorx2 = if (zx & chunk & 0xf0) != 0 { "x" } else { "z" };
                            format!("{}{}", zorx2, zorx1)
                        }
                    })
                    .collect()
            } else {
                drop.take_data()
                    .iter()
                    .rev()
                    .enumerate()
                    .take_while(|(num_chars, _)| *num_chars <= visible_chars / 2)
                    .map(|(_, chunk)| format!("{:x?}", chunk))
                    .collect()
            }
        }

        WaveFormat::Binary => {
            if drop.is_zx() {
                let (payload, zx) = split_zx_and_payload(drop, bitwidth);
                payload
                    .iter()
                    .zip(zx.iter())
                    .rev()
                    .enumerate()
                    .take_while(|(num_chars, _)| *num_chars <= visible_chars / 8)
                    .map(|(idx, (chunk, zx))| match zx {
                        0 => {
                            format!("{:0>8b}", chunk)
                        }
                        _ => {
                            let zx_iter = utils::ZxIter::new(
                                zx.clone().reverse_bits(),
                                chunk.clone().reverse_bits(),
                                if idx == 0 {
                                    (8 - (bitwidth % 8) as u8) % 8
                                } else {
                                    0
                                },
                            );
                            zx_iter.into_iter().collect()
                        }
                    })
                    .collect()
            } else {
                drop.take_data()
                    .iter()
                    .rev()
                    .enumerate()
                    //FIXME: bug
                    .take_while(|(idx, _)| *idx <= bitwidth / 8)
                    .map(|(idx, chunk)| {
                        if idx == 0 {
                            format!("{:b}", chunk)
                        } else {
                            format!("{:0>8b}", chunk)
                        }
                    })
                    .collect()
            }
        }
        _ => {
            unimplemented!()
        }
    };
    println!("gen str is {}", gen_str);
    gen_str.truncate(visible_chars);
    gen_str
}

#[cfg(test)]
#[allow(dead_code, unused_macros, unused_imports, unused_variables)]
mod tests {
    use super::{format_payload, WaveFormat};
    use crate::puddle::testing_utils::test_droplet;
    use test_case::test_case;

    #[test_case([0,0,0xef,0xbe,0xad,0xde], 32, 20, "deadbeef"; "vanilla hex")]
    #[test_case([0,0,0xef,0xbe,0xad,0xde], 32, 4, "dead"; "truncated hex")]
    #[test_case([0,0x80,0xef,0xbe,0xad,0xde], 16, 20, "xxxx"; "xx hex")]
    #[test_case([0,0x80,0xef,0x0b,0xaa,0xfe], 16, 20, "zxxx"; "zx hex")]
    #[test_case([0,0,0x2f, 0x01], 9, 3, "12f"; "small hex")]
    fn hextests<T: Into<Vec<u8>>>(
        content: T,
        bitwidth: usize,
        num_chars: usize,
        baseline: &'static str,
    ) {
        let content: Vec<u8> = content.into();
        let droplet = test_droplet(content.as_slice());
        let output = format_payload(droplet, WaveFormat::Hex, bitwidth, num_chars);
        assert_eq!(output, String::from(baseline));
    }

    #[test_case([0,0,0x2f, 0x01], 9, 9, "100101111"; "vanilla bin")]
    #[test_case([0,0,0x2f, 0x01], 9, 5, "10010"; "truncated bin")]
    #[test_case([0,0x80,0x2f, 0x01], 5, 5, "0111x"; "zz bin")]
    fn bintests<T: Into<Vec<u8>>>(
        content: T,
        bitwidth: usize,
        num_chars: usize,
        baseline: &'static str,
    ) {
        let content: Vec<u8> = content.into();
        let droplet = test_droplet(content.as_slice());
        let output = format_payload(droplet, WaveFormat::Binary, bitwidth, num_chars);
        assert_eq!(output, String::from(baseline));
    }
}
