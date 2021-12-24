use unicode_segmentation::UnicodeSegmentation;

pub struct Unicode {
    pub version: String,
    pub blocks: Vec<Block>,
}
