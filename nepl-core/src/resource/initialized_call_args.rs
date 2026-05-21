use super::cell_state::CellTable;
use super::model::Place;

pub(super) fn discard_call_arg_loaded_value_origins(cells: &mut CellTable, args: &[Place]) {
    for arg in args {
        cells.discard_raw_cell_loaded_value_origin(arg);
    }
}
