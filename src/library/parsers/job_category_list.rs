use anyhow::Result;
use csv::ReaderBuilder;
use std::{collections::BTreeMap, ops::Index, path::Path};

#[derive(Default)]
pub struct JobCategoryList {
    jobs: BTreeMap<u32, Jobs>,
}

#[derive(Default)]
pub struct Jobs {
    pub id: u32,
    pub bits: u32,
}

impl JobCategoryList {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut jobs = BTreeMap::new();

        csv_parse!(path => {
            id = U[0];
            crp = B[9 + 1];
            bsm = B[10 + 1];
            arm = B[11 + 1];
            gsm = B[12 + 1];
            ltw = B[13 + 1];
            wvr = B[14 + 1];
            alc = B[15 + 1];
            cul = B[16 + 1];

            let bits = ((crp as u32) << 0)
                + ((bsm as u32) << 1)
                + ((arm as u32) << 2)
                + ((gsm as u32) << 3)
                + ((ltw as u32) << 4)
                + ((wvr as u32) << 5)
                + ((alc as u32) << 6)
                + ((cul as u32) << 7);

            jobs.insert(id, Jobs { id, bits });
        });

        Ok(Self { jobs })
    }
}

impl Jobs {
    pub fn matches_any(&self, jobs: &Vec<&str>) -> bool {
        let mut bits = 0;
        for job in jobs {
            match &job[..] {
                "crp" | "CRP" => bits |= 1 << 0,
                "bsm" | "BSM" => bits |= 1 << 1,
                "arm" | "ARM" => bits |= 1 << 2,
                "gsm" | "GSM" => bits |= 1 << 3,
                "ltw" | "LTW" => bits |= 1 << 4,
                "wvr" | "WVR" => bits |= 1 << 5,
                "alc" | "ALC" => bits |= 1 << 6,
                "cul" | "CUL" => bits |= 1 << 7,
                _ => panic!("Invalid job: {job}"),
            }
        }
        jobs.is_empty() || (bits & self.bits) > 0
    }
}

impl Index<&u32> for JobCategoryList {
    type Output = Jobs;

    fn index(&self, index: &u32) -> &Self::Output {
        match self.jobs.get(index) {
            None => panic!("Missing job_category id: {index}"),
            Some(value) => value,
        }
    }
}
