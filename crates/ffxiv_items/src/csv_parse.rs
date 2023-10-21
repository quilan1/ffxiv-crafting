macro_rules! parse {
    (@ $info:tt $id:ident = U[$start:literal .. $end:literal]; $($tail:tt)*) => {
        let $id = $info[$start..$end].into_iter().map(|v| v.parse::<u32>().unwrap_or_default()).collect::<Vec<_>>();
        crate::csv_parse::parse!(@ $info $($tail)*)
    };

    (@ $info:tt $id:ident = U[$index:expr]; $($tail:tt)*) => {
        let $id = $info[$index].parse::<u32>()?;
        crate::csv_parse::parse!(@ $info $($tail)*)
    };

    (@ $info:tt $id:ident = S[$index:expr]; $($tail:tt)*) => {
        let $id = $info[$index].to_string();
        crate::csv_parse::parse!(@ $info $($tail)*)
    };

    (@ $info:tt $id:ident = B[$index:expr]; $($tail:tt)*) => {
        let $id = $info[$index] == "True";
        crate::csv_parse::parse!(@ $info $($tail)*)
    };

    (@ $info:tt $($tail:tt)+) => { $($tail)+ };

    (@ $info:tt ) => {}
}

macro_rules! csv_parse {
    ($reader:expr, $info:ident => { $($tail:tt)* }) => {
        let mut reader = csv::ReaderBuilder::new().from_reader($reader);
        for (line, record) in reader.records().enumerate() {
            if line < 2 {
                continue;
            }
            let record = record?;
            let $info = record.into_iter().collect::<Vec<_>>();

            crate::csv_parse::parse!(@ $info $($tail)*);
        }
    };

    ($reader:expr => { $($tail:tt)* }) => { csv_parse!($reader, info => { $($tail)* }); }
}

pub(crate) use {csv_parse, parse};
