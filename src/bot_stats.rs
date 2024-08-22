use std::{cmp::{max, min}, collections::HashMap};

use serde::{Deserialize, Serialize};


#[derive(Deserialize, Debug)]
pub struct BotStats {
    _current: ExecutionStats,
}
#[derive(Deserialize, Debug, Default, Serialize, Clone)]
pub struct QueueStats {
    pub checkpoint: Option<String>,
    pub source_timestamp: i64,
    pub timestamp: i64,
    pub units: u32,
}

impl QueueStats {
    pub fn merge(&mut self, other: &Self) {
        println!("got to merge");
        self.source_timestamp = max(self.source_timestamp, other.source_timestamp);
        self.timestamp = max(self.timestamp, other.timestamp);
        self.units += other.units;
        
        if let Some(other_check) = other.checkpoint.as_ref(){
            self.checkpoint = Some(other_check.to_string())
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum StatsOrEmpty {
    NotEmpty(HashMap<String, QueueStats>),
    Empty {}
}

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct ExecutionStats {
    pub execution: Option<BaseExecutionStats>,
    pub read: StatsOrEmpty,
    pub write: StatsOrEmpty
}

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct BaseExecutionStats {
    pub completions: Option<u32>,
    pub duration: Option<u32>,
    pub errors: Option<u32>,
    pub max_duration: Option<u32>,
    pub min_duration: Option<u32>,
    pub units: Option<u32>
}

#[derive(Deserialize, Debug, Serialize)]
pub struct Stats {
    completions: u32,
    duration: u32,
    errors: u32,
    max_duration: u32,
    min_duration: u32,
    units: u32
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CondensedStats {
    execution_stats: Stats,
    read: HashMap<String, QueueStats>,
    write: HashMap<String, QueueStats>,
}

impl CondensedStats {
    pub fn merge_execution_stats(&mut self, other: &DynamoStatsRecord) {
        if let Some(other_execution_stats) = other.current.execution.as_ref() {
            if let Some(o_completions) = other_execution_stats.completions {
                self.execution_stats.completions += o_completions;
            }
            if let Some(o_units) = other_execution_stats.units {
                self.execution_stats.units += o_units;
            }
            if let Some(o_duration) = other_execution_stats.duration {
                self.execution_stats.duration += o_duration;
            }
            if let Some(o_max_duration) = other_execution_stats.max_duration {
                self.execution_stats.max_duration = max(self.execution_stats.max_duration, o_max_duration);
            }
            if let Some(o_errors) = other_execution_stats.errors {
                self.execution_stats.errors += o_errors;
            }
            
            if let Some(o_min_duration) = other_execution_stats.min_duration {
                if o_min_duration > 0 {
                    self.execution_stats.min_duration = min(self.execution_stats.min_duration, o_min_duration);
                }
            }
        }
        
        self.merge_stats(&other.current.read, &other.current.write)
        
    }
    
    pub fn merge_stats(&mut self, other_read: &StatsOrEmpty, other_write: &StatsOrEmpty) {
        println!("got to merge_stats");
        
        // Merge per queue as we will want reporting per queue
        if let StatsOrEmpty::NotEmpty(o_read) = other_read {
            
            // Loop through o_read and merge into self.read
            
            if !o_read.is_empty() {
                for (queue_id, o_stat) in o_read {

                    self.read.entry(queue_id.to_owned())
                        .and_modify(|e| e.merge(o_stat))
                        .or_insert(o_stat.clone());
                }
            }
        }
        
        if let StatsOrEmpty::NotEmpty(o_write) = other_write {
            if !o_write.is_empty() {
                for (queue_id, o_stat) in o_write {
                    self.write.entry(queue_id.to_owned())
                        .and_modify(|e| e.merge(o_stat))
                        .or_insert(o_stat.clone());
                }
            }
        }
        
        
    }
    
}


#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct DynamoStatsRecord {
    pub id: String,
    pub bucket: String,
    pub current: ExecutionStats,
    pub period: String, // Eventually this will be an enum
    // previous: ExecutionStats,
    pub start_eid: Option<String>,
    pub time: i64
}

/// Merges all of the bot_stats into a single instance of CondensedStats
pub fn merge_bot_stats(bot_stats: &[DynamoStatsRecord]) -> CondensedStats {
    
   let mut stat = CondensedStats {
        execution_stats: Stats {
            completions: 0,
            duration: 0,
            errors: 0,
            max_duration: 0,
            min_duration: 0,
            units: 0,
        },
        read: HashMap::new(),
        write: HashMap::new(),
        
   };
   
   for stats in bot_stats {
       stat.merge_execution_stats(&stats)
   }
   stat
}


#[cfg(test)]
mod bot_stats_tests {
    use std::fs::read_to_string;

    use super::{merge_bot_stats, DynamoStatsRecord};

    #[test]
    fn combinor_works() {
        
        let bot_stats_raw = read_to_string("./bot_stats.json").unwrap();
        
        let stats: Vec<DynamoStatsRecord> = serde_json::from_str(&bot_stats_raw).unwrap();
        
        let combined = merge_bot_stats(&stats);
        
        println!("{}", serde_json::to_string(&combined).unwrap());
    }
}