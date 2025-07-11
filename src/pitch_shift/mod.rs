pub mod psola;

use nih_plug::prelude::Enum;

#[derive(Debug, PartialEq, Enum)]
pub enum PitchShiftKind {
    Classic,
    PSOLA,
}
