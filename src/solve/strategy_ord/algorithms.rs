use itertools::Itertools;

use crate::layout::*;
use crate::puzzle::*;

macro_rules! export {
    ($module:ident => $($item:ident),+ $(,)?) => {
        mod $module;
        pub use $module::{ $($item),+ };
    };
}

export!(avoidable_rectangles => find_avoidable_rectangles);
export!(brute_force => find_brute_force, BruteForceResult);
export!(bugs => find_bugs);
export!(empty_rectangles => find_empty_rectangles);
export!(fireworks => find_fireworks);

export!(
    fish =>
        find_jellyfish,
        find_swordfish,
        find_x_wings
);

export!(hidden_singles => find_hidden_singles);

export!(
    hidden_tuples =>
        find_hidden_pairs,
        find_hidden_triples,
        find_hidden_quads
);

export!(intersection_removals => find_intersection_removals);
export!(naked_singles => find_naked_singles);

export!(
    naked_tuples =>
        find_naked_pairs,
        find_naked_triples,
        find_naked_quads
);

export!(peers => find_peers);
export!(singles_chains => find_singles_chains);
export!(skyscrapers => find_skyscrapers);
export!(two_string_kites => find_two_string_kites);
export!(unique_rectangles => find_unique_rectangles);
export!(wxyz_wings => find_wxyz_wings);
export!(xy_chains => find_xy_chains);
export!(xyz_wings => find_xyz_wings);
export!(y_wings => find_y_wings);
