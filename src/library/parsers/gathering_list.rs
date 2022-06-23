use anyhow::Result;
use csv::ReaderBuilder;
use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufWriter, Write},
    ops::Index,
    path::Path,
};

use crate::{
    library::Library, market_board_analysis::MarketBoardAnalysis, universalis::Universalis,
    Settings,
};

#[derive(Default)]
pub struct GatheringList {
    by_item: BTreeMap<u32, Vec<u32>>,
    gathering: BTreeMap<u32, GatheringInfo>,
}

pub struct GatheringInfo {
    pub id: u32,
    pub item_id: u32,
    pub level: u32,
}

impl GatheringList {
    pub fn from_path<P: AsRef<Path>>(path: P, library: &Library) -> Result<Self> {
        let mut gathering = BTreeMap::new();
        let mut by_item = BTreeMap::<u32, Vec<u32>>::new();

        let mut reader = ReaderBuilder::new().from_path(path)?;
        for (line, record) in reader.records().enumerate() {
            if line < 2 {
                continue;
            }

            let record = record?;
            let info = record.into_iter().collect::<Vec<_>>();

            let id = info[0].parse::<u32>()?;
            let item_id = info[0 + 1].parse::<u32>()?;
            let level = info[1 + 1].parse::<u32>()?;
            let level = library.all_gathering_levels[&level];

            if !library.all_items.items.contains_key(&item_id) {
                continue;
            }

            if library.all_items[&item_id].name == "" {
                continue;
            }

            gathering.insert(id, GatheringInfo { id, item_id, level });

            by_item.entry(item_id).or_default().push(id);
        }

        Ok(Self {
            by_item: by_item,
            gathering: gathering,
        })
    }

    pub fn contains_item_id(&self, item_id: &u32) -> bool {
        self.by_item.contains_key(item_id)
    }

    pub fn write_to_file<P: AsRef<Path>>(
        &self,
        path: P,
        library: &Library,
        universalis: &Universalis,
        settings: &Settings,
    ) -> Result<()> {
        let mut writer = BufWriter::new(File::create(path.as_ref())?);

        write!(&mut writer, "{:<40}| {:<30}| {:<10}\n", "Name", "Vel", "Sell")?;
        write!(
            &mut writer,
            "=========================================================================================\n"
        )?;

        let mut analyses = self
            .gathering
            .iter()
            .filter_map(|(_, item)| {
                MarketBoardAnalysis::from_item(item.item_id, &universalis, &settings)
            })
            .collect::<Vec<_>>();
        analyses.sort_by_key(|analysis| analysis.sell_price);

        for analysis in analyses {
            if analysis.sell_price < settings.min_gathering_price {
                continue;
            }

            if analysis.velocity_info_nq.velocity < settings.min_gathering_velocity {
                continue;
            }

            let item = &library.all_items[&analysis.item_id];
            write!(
                &mut writer,
                "{:<40}| {:<30}| {:<10}\n",
                item.name, analysis.velocity_info_nq.to_string(), analysis.sell_price
            )?;
        }
        Ok(())
    }
}

impl Index<&u32> for GatheringList {
    type Output = GatheringInfo;

    fn index(&self, index: &u32) -> &Self::Output {
        match self.gathering.get(index) {
            None => panic!("Missing gathering id: {index}"),
            Some(value) => &value,
        }
    }
}
