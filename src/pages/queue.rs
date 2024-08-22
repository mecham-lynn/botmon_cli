use std::collections::HashMap;

use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use ratatui::widgets::ScrollbarState;
use serde::Serialize;
use tui_input::Input;

use crate::{
    bot_stats::{DynamoStatsRecord, QueueStats, Stats, StatsOrEmpty},
    dynamo::QueueDetails,
};

#[derive(Debug, Default, Serialize)]
pub struct QueueSearchState {
    pub all_queues: Option<Vec<QueueDetails>>,
    pub current_select_index: usize,
    pub queues: Vec<String>,
    pub search: Input,
    pub search_results: Vec<String>,
    pub selected_queue_name: Option<String>,
    pub selected_queue: Option<QueueViewState>,
    pub stats: Vec<DynamoStatsRecord>,
}

#[derive(Debug, Serialize)]
pub struct QueueViewState {
    pub vertical_scroll_state: ScrollbarState,
    pub vertical_scroll: usize,
    pub full_stats: Vec<DynamoStatsRecord>,
    pub write_stats: HashMap<String, Vec<QueueStats>>,
    pub read_stats: HashMap<String, Vec<QueueStats>>,
}

impl QueueViewState {
    fn write_stats_from_all_stats(stats: &[DynamoStatsRecord]) -> HashMap<String, Vec<QueueStats>> {
        let mut write_stats = HashMap::new();
        
        stats.iter().for_each(|a| {
            if let StatsOrEmpty::NotEmpty(bot_stats) = &a.current.write {
                bot_stats.iter().for_each(|(bot_id, stat)| {
                   write_stats.entry(bot_id.to_owned())
                       .and_modify(|bot_data: &mut Vec<QueueStats>| bot_data.push(stat.clone()))
                       .or_insert(vec![stat.clone()]);
                });
            }
        });
        
        write_stats
    }
    
    fn read_stats_from_all_stats(stats: &[DynamoStatsRecord]) -> HashMap<String, Vec<QueueStats>> {
        let mut read_stats = HashMap::new();
        
        stats.iter().for_each(|a| {
            if let StatsOrEmpty::NotEmpty(bot_stats) = &a.current.read {
                bot_stats.iter().for_each(|(bot_id, stat)| {
                   read_stats.entry(bot_id.to_owned())
                       .and_modify(|bot_data: &mut Vec<QueueStats>| bot_data.push(stat.clone()))
                       .or_insert(vec![stat.clone()]);
                });
            }
        });
        
        read_stats
    }
}

impl QueueSearchState {
    pub fn queue_names(&mut self) {
        self.queues = self
            .all_queues
            .as_ref()
            .unwrap()
            .iter()
            .map(|queue| queue.event.replace("queue:", ""))
            .collect()
    }
    
    //FIXME: Duplicate with BotPageState::search_bots() make this into a trait
    pub fn search_queues(&mut self) {
        let mut matches = vec![];
        let value = self.search.value();
        let matcher = SkimMatcherV2::default();
        
        for id in &self.queues {
            if let Some(match_score) = matcher.fuzzy_match(id, value) {
                matches.push((id, match_score))
            }
        }
        
        // Sort the matches by score descending
        matches.sort_by(|a, b| b.1.cmp(&a.1));
        
        self.search_results = matches.iter().map(|a| a.0.to_owned()).collect();
    }
    

}
