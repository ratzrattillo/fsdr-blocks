use futuresdr::prelude::*;

use crate::cw::shared::CWAlphabet::{self, LetterSpace, WordSpace};
use crate::cw::shared::get_alphabet;
use bimap::BiMap;

#[derive(Block)]
pub struct CWToChar<
    I: CpuBufferReader<Item = CWAlphabet> = DefaultCpuReader<CWAlphabet>,
    O: CpuBufferWriter<Item = u32> = DefaultCpuWriter<u32>,
> {
    #[input]
    input: I,
    #[output]
    output: O,
    // Required to keep the state of already received pulses
    symbol_vec: Vec<CWAlphabet>,
    alphabet: BiMap<char, Vec<CWAlphabet>>,
}

impl<I, O> CWToChar<I, O>
where
    I: CpuBufferReader<Item = CWAlphabet>,
    O: CpuBufferWriter<Item = u32>,
{
    pub fn new(alphabet: BiMap<char, Vec<CWAlphabet>>) -> Self {
        CWToChar {
            input: I::default(),
            output: O::default(),
            symbol_vec: vec![],
            alphabet,
        }
    }
}

#[doc(hidden)]
impl<I, O> Kernel for CWToChar<I, O>
where
    I: CpuBufferReader<Item = CWAlphabet>,
    O: CpuBufferWriter<Item = u32>,
{
    async fn work(
        &mut self,
        io: &mut WorkIo,
        _mio: &mut MessageOutputs,
        _meta: &mut BlockMeta,
    ) -> Result<()> {
        let i = self.input.slice();
        let o = self.output.slice();

        let (consumed, produced, finished) = if i.is_empty() {
            (0, 0, self.input.finished())
        } else {
            // Not doing any checks on the output buffer length here.
            // Assuming, that i and o are of the same length.
            // Assuming, that one input sample generates at max one output sample.
            self.symbol_vec.append(&mut i.to_vec());

            let mut produced = 0;
            if self.symbol_vec.contains(&WordSpace) || self.symbol_vec.contains(&LetterSpace) {
                let symbols: Vec<_> = self
                    .symbol_vec
                    .split_inclusive(|c| c == &LetterSpace || c == &WordSpace)
                    .filter_map(|c| c.split_last())
                    .map(|(last, elements)| {
                        //println!("last: {}, elements: {:?}", last, elements);
                        if last == &WordSpace {
                            *self.alphabet.get_by_right(&vec![WordSpace]).unwrap_or(&'_')
                        } else {
                            *self.alphabet.get_by_right(elements).unwrap_or(&'_')
                        }
                    })
                    .collect();

                let n = std::cmp::min(symbols.len(), o.len());
                for j in 0..n {
                    o[j] = symbols[j] as u32;
                    //println!("c: {}, index: {}, produced: {}", c, index, produced);
                }
                produced = n;
                self.symbol_vec.clear(); // This might be wrong if we didn't consume everything, but original did this
            }

            (i.len(), produced, self.input.finished())
        };

        if consumed > 0 {
            self.input.consume(consumed);
        }
        if produced > 0 {
            self.output.produce(produced);
        }
        if finished {
            io.finished = true;
        }

        Ok(())
    }
}

pub struct CWToCharBuilder {
    alphabet: BiMap<char, Vec<CWAlphabet>>,
}

impl Default for CWToCharBuilder {
    fn default() -> Self {
        CWToCharBuilder {
            alphabet: get_alphabet(),
        }
    }
}

impl CWToCharBuilder {
    pub fn new() -> CWToCharBuilder {
        CWToCharBuilder::default()
    }

    /*pub fn alphabet(mut self, alphabet: BiMap<char, Vec<CWAlphabet>>) -> CWToCharBuilder {
        self.alphabet = alphabet;
        self
    }*/

    pub fn build(self) -> CWToChar {
        CWToChar::new(self.alphabet)
    }
}
