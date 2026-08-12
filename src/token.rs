use std::sync::OnceLock;
use tiktoken_rs::{CoreBPE, cl100k_base};

pub fn tokens(text: &str) -> usize {
    static BPE: OnceLock<CoreBPE> = OnceLock::new();
    let bpe = BPE.get_or_init(|| {
        cl100k_base().expect("cl100k tokenizer data failed to load — corrupt build")
    });
    bpe.encode_with_special_tokens(text).len()
}
