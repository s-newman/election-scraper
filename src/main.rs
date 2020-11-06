use std::collections::HashMap;
use std::str;

use serde::{Deserialize, Serialize};

type VoteCount = u32;

#[derive(Serialize, Deserialize)]
struct Results {
    data: Data,
}

#[derive(Serialize, Deserialize)]
struct Data {
    races: Vec<Race>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Race {
    state_id: String,
    state_slug: String,
    state_name: String,
    votes: VoteCount,
    electoral_votes: u8,
    eevp: u8,
    tot_exp_vote: VoteCount,
    last_updated: String,
    candidates: Vec<Candidate>,
    counties: Vec<County>,
    expectations_text: Option<String>,
    timeseries: Vec<Timeseries>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Candidate {
    candidate_key: String,
    first_name: String,
    last_name: String,
    name_display: String,
    votes: VoteCount,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct County {
    fips: String,
    name: String,
    votes: VoteCount,
    eevp: Option<u8>,
    tot_exp_vote: Option<VoteCount>,
    last_updated: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Timeseries {
    vote_shares: HashMap<String, f64>,
    votes: VoteCount,
    eevp: u8,
    timestamp: String,
}

fn main() {
    // Grab the data and make sure we got something
    let resp = ureq::get(
        "https://static01.nyt.com/elections-assets/2020/data/api/2020-11-03/national-map-page/national/president.json"
    )
    .call();
    if !resp.ok() {
        println!("Request failed.");
        return;
    }

    // Deserialize and store the data
    let resp_str = resp.into_string().unwrap();
    let jd = &mut serde_json::Deserializer::from_str(&resp_str);

    let results: Results = match serde_path_to_error::deserialize(jd) {
        Ok(r) => r,
        Err(e) => {
            let path = e.path().to_string();
            println!("Error at {}:\n{}", path, e);
            return;
        }
    };

    for state in results.data.races {
        let remaining: i32 = state.tot_exp_vote as i32 - state.votes as i32;
        if remaining < 0 {
            continue;
        }
        let biden = state
            .candidates
            .iter()
            .filter(|c| c.candidate_key == "bidenj")
            .collect::<Vec<&Candidate>>()[0];
        let trump = state
            .candidates
            .iter()
            .filter(|c| c.candidate_key == "trumpd")
            .collect::<Vec<&Candidate>>()[0];

        let gap: i32 = trump.votes as i32 - biden.votes as i32;

        let margin = gap as f64 / remaining as f64;
        if margin.abs() < 0.3 {
            println!("{}: {:.02}%", state.state_name, margin * 100.0);
        }
    }
}
