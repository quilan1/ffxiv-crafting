macro_rules! parse {
    // (@ $info:tt $id:ident = R[$start:expr,$end:expr]; $($tail:tt)*) => {
    //     let $id = $info[$start..$end].into_iter().map(|v| v.parse::<u32>().unwrap());
    //     parse!(@ $info $($tail)*)
    // };

    (@ $info:tt $id:ident = U[$index:expr]; $($tail:tt)*) => {
        let $id = $info[$index].parse::<u32>()?;
        parse!(@ $info $($tail)*)
    };

    (@ $info:tt $id:ident = S[$index:expr]; $($tail:tt)*) => {
        let $id = $info[$index].to_string();
        parse!(@ $info $($tail)*)
    };

    (@ $info:tt $id:ident = B[$index:expr]; $($tail:tt)*) => {
        let $id = $info[$index] == "True";
        parse!(@ $info $($tail)*)
    };

    (@ $info:tt $($tail:tt)*) => { $($tail)* };
}

#[macro_export]
macro_rules! csv_parse {
    ($path:expr, $info:ident => { $($tail:tt)* }) => {
        let mut reader = ReaderBuilder::new().from_path($path)?;
        for (line, record) in reader.records().enumerate() {
            if line < 2 {
                continue;
            }
            let record = record?;
            let $info = record.into_iter().collect::<Vec<_>>();

            parse!(@ $info $($tail)*);
        }
    };

    ($path:expr => { $($tail:tt)* }) => { csv_parse!($path, info => { $($tail)* }); }
}

#[macro_use]

mod craft_leve_list;
mod gathering_level_list;
mod gathering_list;
mod item_list;
mod job_category_list;
mod leve_list;
mod recipe_level_list;
mod recipe_list;
mod ui_category_list;

pub use craft_leve_list::*;
pub use gathering_level_list::*;
pub use gathering_list::*;
pub use item_list::*;
pub use job_category_list::*;
pub use leve_list::*;
pub use recipe_level_list::*;
pub use recipe_list::*;
pub use ui_category_list::*;
