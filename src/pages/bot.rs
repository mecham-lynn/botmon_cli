use std::{collections::HashMap, fs::File, io::Write};

use chrono::Duration;
use color_eyre::eyre::{bail, Result};
use crossterm::event::{Event, KeyCode, KeyEvent};
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use ratatui::widgets::ScrollbarState;
use serde::{Deserialize, Serialize};
use tui_input::{backend::crossterm::EventHandler, Input};
use std::fs::read_to_string;

use crate::{app::Navigate, bot_stats::{BotDynamoStatsRecord, BotStats, QueueStats, StatsOrEmpty}};

#[derive(Debug)]
pub struct BotViewState {
   pub vertical_scroll_state: ScrollbarState,
   pub vertical_scroll: usize,
   pub setting: BotSettings,
   pub full_stats: Vec<BotDynamoStatsRecord>,
   pub write_stats: HashMap<String, Vec<QueueStats>>,
   pub read_stats: HashMap<String, Vec<QueueStats>>,
   // pub read_connections: Vec<Connection>,
   // pub write_connections: Vec<Connection>
}

#[derive(Debug)]
pub struct Connection {
    name: String,
    num_actions: u32,
}

impl BotViewState {
    pub fn new(setting: BotSettings, stats: Vec<BotDynamoStatsRecord>) -> Self {
        Self {
            vertical_scroll_state: ScrollbarState::default(),
            vertical_scroll: 0,
            setting,
            write_stats: Self::write_stats_from_all_stats(&stats),
            read_stats: Self::read_stats_from_all_stats(&stats),
            full_stats: stats,
        }
    }
    
    fn write_stats_from_all_stats(stats: &[BotDynamoStatsRecord]) -> HashMap<String, Vec<QueueStats>> {
        let mut write_stats = HashMap::new();
        
        stats.iter().for_each(|a| {
            if let StatsOrEmpty::NotEmpty(queue_stats) = &a.current.write {
                queue_stats.iter().for_each(|(queue_id, stat)| {
                   write_stats.entry(queue_id.to_owned())
                       .and_modify(|queue_data: &mut Vec<QueueStats>| queue_data.push(stat.clone()))
                       .or_insert(vec![stat.clone()]);
                });
            }
        });
        
        write_stats
    }
    
    fn read_stats_from_all_stats(stats: &[BotDynamoStatsRecord]) -> HashMap<String, Vec<QueueStats>> {
        let mut read_stats = HashMap::new();
        
        stats.iter().for_each(|a| {
            if let StatsOrEmpty::NotEmpty(queue_stats) = &a.current.read {
                queue_stats.iter().for_each(|(queue_id, stat)| {
                   read_stats.entry(queue_id.to_owned())
                       .and_modify(|queue_data: &mut Vec<QueueStats>| queue_data.push(stat.clone()))
                       .or_insert(vec![stat.clone()]);
                });
            }
        });
        
        read_stats
    }
    
}

impl Navigate for BotViewState {
    fn navigate(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        match key_event.code {
            KeyCode::Up => {
                self.vertical_scroll = self.vertical_scroll.saturating_sub(1);
                self.vertical_scroll_state = self.vertical_scroll_state.position(self.vertical_scroll);
            }
            KeyCode::Down => {
                self.vertical_scroll = self.vertical_scroll.saturating_add(1);
                self.vertical_scroll_state = self.vertical_scroll_state.position(self.vertical_scroll);
            }
            a => {
                bail!("invalid key {a:?} pressed");
            }
        }
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct BotPageState {
    pub bots: Vec<String>,
    pub stats: Vec<BotDynamoStatsRecord>, // String for now will need to be a type
    pub selected_bot_name: Option<String>,
    pub current_select_index: usize,
    pub selected_bot: Option<BotViewState>,
    pub all_bots: Option<Vec<BotSettings>>,
    pub search: Input,
    pub search_results: Vec<String>,
    // settings: 
}

// impl Default for BotPageState {
//     fn default() -> Self {
//         Self::new()
//     }
// }

impl BotPageState {

    pub fn bot_names(&mut self) {
       self.bots = self.all_bots.as_ref().unwrap().iter().map(|bot| bot.id.replace("bot:", "")).collect()
    }
    // TODO update this to search for the tmp storage first
    // then to hit dynamo when the date of the tmp storage > 5 min
    // currently we just read a large json file
    // pub fn new( bot_settings: Vec<BotSettings>) -> Self {
    //     // let data = read_to_string("./all_bot_settings.json").unwrap();
    //     // let settings: Vec<BotSettings> = serde_json::from_str(&data).unwrap();
    //     let bot_names: Vec<String> = bot_settings.iter().map(|bot| bot.id.replace("bot:", "")).collect();
        
    //     Self {
    //         bots: bot_names,
    //         stats: vec![],
    //         selected_bot_name: None,
    //         selected_bot: None,
    //         all_bots: Some(bot_settings),
    //         search: Input::default(),
    //         search_results: vec![],
    //         current_select_index: 0,
    //     }
    //     // let data: BotSettings = read_to
    // }
    
    pub fn search_bots(&mut self) {
        let mut matches = vec![];
        let value = self.search.value();
        let matcher = SkimMatcherV2::default();
        
        // fuzzy match on the bot names
        for name in &self.bots {
            if let Some(match_score) = matcher.fuzzy_match(name, value) {
                matches.push((name, match_score))
            }
        }
        
        // Sort the matches by score descending
        matches.sort_by(|a, b| b.1.cmp(&a.1));
        
        self.search_results = matches.iter().map(|a| a.0.to_owned()).collect();
    }
    
    pub fn get_bot_details(&mut self)-> Result<()> {
        match (self.all_bots.as_ref(), self.selected_bot_name.as_ref()) {
            (None, None) => bail!("no bots loaded AND no bot selected"),
            (None, Some(_)) => bail!("no bots loaded"),
            (Some(_), None) => bail!("no bot selected when attempting to get bot details"),
            (Some(all_bots), Some(selected)) => {
                // Load bots stats from file. Eventually this will be loaded directly from dynamo
                // let all_stats = read_to_string("./all_bot_stats.json")?;
                // let stats: Vec<BotDynamoStatsRecord> = serde_json::from_str(&all_stats)?;
                
                let bot_stats: Vec<BotDynamoStatsRecord> = self.stats.iter()
                    .filter(|a| a.id.contains(selected) && a.period == "minute_15")
                    .cloned()
                    .collect();
                let settings = match all_bots.iter().find(|&a| a.id.contains(selected)).cloned() {
                    Some(a) => a,
                    None => bail!("unable to locate settings for selected bot '{selected}'"),
                };
                
                //REMOVE THE BELOW
                // let stats_filename = format!("./{}.json", selected);
                // let mut file = File::create(&stats_filename)?;
                // file.write_all(serde_json::to_string(&bot_stats)?.as_bytes())?;
                // DON'T REMOVE BELOW
                self.selected_bot = Some(BotViewState::new(settings, bot_stats));
            },
        };
        
       
        
        Ok(())
    }
}

// impl Navigate for BotPageState {
//     fn navigate(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        
//         let list_len =  if self.search_results.is_empty() {
//             0_usize
//         } else {
//             self.search_results.len()
//         };
        
//         match key_event.code {
//             KeyCode::Down => {
//                 if list_len > 0 {
//                     let index  = self.current_select_index + list_len;
//                     self.current_select_index = index.saturating_add(1) % list_len;
//                 }
//             },
//             KeyCode::Up => {
//                 if list_len > 0 {
//                     let index  = self.current_select_index + list_len;
//                     self.current_select_index = index.saturating_sub(1) % list_len;
//                 }
//             }, 
//             KeyCode::Enter => {
//                 if list_len > 0 {
//                     self.selected_bot_name = Some(self.search_results[self.current_select_index].clone());
//                     self.search.reset();
//                     self.search_results.clear()
//                 }
//                 //TODO after we select a bot switch views to the BotView and clear the search
//             }
//             _ => {
//                 // Do the search
//                 self.search.handle_event(&Event::Key(key_event));
//                 self.search_bots()
//             }
            
//         }
        
        
        
        
        
//         Ok(())
//     }
// }


#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all="camelCase")]
pub struct BotSettings {
    pub id: String,
    checkpoints: Option<Checkpoints>,
    description: Option<String>,
    error_count: Option<u32>,
    execution_type: Option<String>,
    instances: Option<HashMap<String, Instance>>,
    invoke_time: Option<i64>,
    lambda_name: Option<String>,
    message: Option<String>,
    name: Option<String>,
    paused: Option<bool>,
    progress: Option<HashMap<String, String>>,
    #[serde(rename="requested_kinesis")]
    requested_kinesis: Option<HashMap<String, String>>,
    scheduled_trigger: Option<i64>,
    tags: Option<String>, // comma-delimited-list
    token: Option<i64>,
    trigger: Option<i64>,
    triggers: Option<Vec<String>>,
    #[serde(rename="type")]
    r_type: Option<String>, // Eventual enum
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Checkpoints {
    read: Option<HashMap<String, CheckpointDetail>>,
    write: Option<HashMap<String, CheckpointDetail>>
}
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all="snake_case")]
pub struct CheckpointDetail {
    checkpoint: Option<StrOrNum>,
    ended_timestamp: Option<StrOrNum>,
    records: Option<u32>,
    source_timestamp: Option<StrOrNum>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all="camelCase")]
pub struct Instance {
    completed_time: Option<i64>,
    invoke_time: Option<i64>,
    // Will need to be unzipped
    // log: Vec<u8>
    max_duration: Option<u32>,
    request_id: Option<StrOrNum>,
    result: Option<String>,
    start_time: Option<i64>,
    status: Option<String>,
    token: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all="camelCase")]
pub struct Lambda {
    settings: Vec<HashMap<String, String>>
}
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum StrOrNum {
    String(String),
    Num(i64)
    
}