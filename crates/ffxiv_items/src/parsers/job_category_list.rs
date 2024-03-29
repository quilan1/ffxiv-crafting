use anyhow::Result;
use csv::ReaderBuilder;
use std::{collections::BTreeMap, io::Read, ops::Index};

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
    pub fn from_reader<R: Read>(reader: R) -> Result<Self> {
        let mut jobs = BTreeMap::new();

        csv_parse!(reader => {
            id = U[0];
            crp = B[9 + 1];
            bsm = B[10 + 1];
            arm = B[11 + 1];
            gsm = B[12 + 1];
            ltw = B[13 + 1];
            wvr = B[14 + 1];
            alc = B[15 + 1];
            cul = B[16 + 1];

            let bits = u32::from(crp)
                + (u32::from(bsm) << 1)
                + (u32::from(arm) << 2)
                + (u32::from(gsm) << 3)
                + (u32::from(ltw) << 4)
                + (u32::from(wvr) << 5)
                + (u32::from(alc) << 6)
                + (u32::from(cul) << 7);

            jobs.insert(id, Jobs { id, bits });
        });

        Ok(Self { jobs })
    }
}

impl Jobs {
    pub fn matches_any<S: AsRef<str>>(&self, jobs: &[S]) -> bool {
        let mut bits = 0;
        for job in jobs {
            let job = job.as_ref();
            match job {
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
