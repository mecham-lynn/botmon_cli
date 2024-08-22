use std::fmt::Display;

use aws_sdk_dynamodb::{types::{AttributeValue, ReturnConsumedCapacity}, Client};
use chrono::{DateTime, Duration, Utc};
use color_eyre::eyre::{bail, Context};
use serde::{Deserialize, Serialize};
use serde_dynamo::from_item;
use serde_json::Value;

use crate::{bot_stats::DynamoStatsRecord, pages::bot::BotSettings};

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all="snake_case")]
pub enum Period {
    Minute,
    Minute5,
    Minute15,
    Hour,
    Day,
    Week,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct QueueDetails {
    pub archive: Option<bool>,
    #[serde(alias = "_clone")]
    pub clone: Option<String>,
    pub event: String, 
    pub max_eid: Option<String>,
    pub timestamp: Option<i64>,
    pub v: Option<u32>,
}

impl Display for Period {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Period::Minute => write!(f,"minute"),
            Period::Minute5 => write!(f, "minute_5"),
            Period::Minute15 => write!(f,"minute_15"),
            Period::Hour => write!(f,"hour"),
            Period::Day => write!(f,"day"),
            Period::Week => write!(f, "week")
        }
    }
}

#[derive(Debug)]
pub struct AllBuckets {
    period: Period,
    start: i64,
    end: i64
}

#[derive(Debug, Clone, Copy)]
pub struct AllBucketsBuilder {
    period: Period,
    past_ms: i64,
    search_til: DateTime<Utc>
}

impl AllBucketsBuilder {
    pub fn new(period: Period) -> Self {
        let now = Utc::now();
        let fifteen_min = Duration::minutes(15).num_milliseconds();
        Self {
            period,
            past_ms: fifteen_min,
            search_til: now,
        }
    }
    
    
    pub fn past_ms(&mut self, duration: Duration) -> &mut Self {
        
        self.past_ms = duration.num_milliseconds();
        
        self
    }
    
    pub fn search_til(&mut self, date: DateTime<Utc>) -> &mut Self {
        self.search_til = date;
        self
    }
    
    pub fn build(self) -> AllBuckets {     
        AllBuckets {
            period: self.period,
            start: (self.search_til - Duration::milliseconds(self.past_ms)).timestamp_millis(),
            end: self.search_til.timestamp_millis(),
        }
    }
}

#[derive(Debug)]
pub struct BotBucket {
    period: Period,
    date: Option<DateTime<Utc>>
}

impl BotBucket {
    pub fn new(period: Period, date: Option<DateTime<Utc>>) -> Self {
        Self {
            period,
            date,
        }
    }
}

impl Display for BotBucket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.date {
            Some(date) => {
                let date = match self.period {
                    Period::Minute | Period::Minute15 | Period::Minute5=> {
                        date.format("%Y-%m-%d %H:%M").to_string()
                    },
                    Period::Hour => {
                        date.format("%Y-%m-%d %H").to_string()
                    },
                    Period::Day => {
                        date.format("%Y-%m-%d").to_string()
                    },
                    Period::Week => unimplemented!("haven't found an instance of a week yet"),
                };
                
                write!(f, "{}_{date}", self.period)
            },
            None => {
                write!(f, "{}__", self.period)
            },
        }
        
    }
}

pub async fn get_all_bot_stats_for_period(client: &Client, table_name: &str, bucket: AllBuckets) -> color_eyre::Result<Vec<DynamoStatsRecord>> {
    // println!("bucket = {bucket:?}");
    let mut stats: Vec<DynamoStatsRecord> = vec![];
    
   //  let query = {
			// 	TableName: STATS_TABLE,
			// 	IndexName: "period-time-index",
			// 	KeyConditionExpression: "#period = :period and #time between :start and :end",
			// 	ExpressionAttributeNames: {
			// 		"#time": "time",
			// 		"#period": "period"
			// 	},
			// 	ExpressionAttributeValues: {
			// 		":start": span.start,
			// 		":end": span.end,
			// 		":period": period
			// 	},
			// 	"ReturnConsumedCapacity": 'TOTAL'
			// };
    
    let output = client.query()
        .table_name(table_name)
        .index_name("period-time-index")
        .key_condition_expression("#period = :period and #time between :start and :end")
        .expression_attribute_names("#time", "time")
        .expression_attribute_names("#period", "period")
        .expression_attribute_values(":start", AttributeValue::N(bucket.start.to_string()))
        .expression_attribute_values(":end", AttributeValue::N(bucket.end.to_string()))
        .expression_attribute_values(":period", AttributeValue::S(bucket.period.to_string()))
        .send().await.wrap_err_with(||format!("failed to get all stats|{table_name}|{bucket:?}"))?;
    
    if output.items.is_none() {
        bail!("no items were returned from dynamo query")
    }
    
    let items = output.items();
    
    // println!("query returned {} items", items.len());
    
    for (_index, item) in items.iter().enumerate() {
        stats.push(match from_item(item.clone()) {
            Ok(a) => a,
            Err(e) => {
                let json: Value = from_item(item.clone()).unwrap();
                bail!("failed to deserialize: '{e}' \n {}", serde_json::to_string(&json).unwrap())
            },
        });
    }
    
    Ok(stats)
}

pub async fn get_all_bot_details(client: &Client, table_name: &str)-> color_eyre::Result<Vec<BotSettings>> {
    let page_size = 100;
    let mut bots: Vec<BotSettings> = vec![];
    
    let items: Result<Vec<_>, _>  = client
        .scan()
        .table_name(table_name)
        .limit(page_size)
        .into_paginator()
        .items()
        .send()
        .collect()
        .await;
    let raw_items = items.wrap_err("failed getting bot settings")?;
    for item in &raw_items {
        bots.push(match from_item(item.clone()) {
            Ok(a) => a,
            Err(e) => {
                // let json: Value = from_item(item.clone()).unwrap();
                bail!("failed to deserialize: '{e}' \n {:?}", item);
            }
        })
    }
    
    Ok(bots)
}


/// Queries the dynamo table for all stats in range for a given bot_id.
/// We HAVE to do a BETWEEN dynamo call and we want to sort descending as well. Which period we want to search should be a param we pass in
/// 
pub async fn get_stats_from_time(client: &Client, bot_id: &str ,table_name: &str, start_bucket: BotBucket, end_bucket: BotBucket) -> color_eyre::Result<Vec<DynamoStatsRecord>> {
    
    
    // println!("bucket = {start_bucket}");
    // println!("end_bucket = {end_bucket:#?}");
    // println!("bot_id = {bot_id}");
    
    let mut stats = vec![];
    let output = client.query()
        .table_name(table_name)
        // .index_name("period-time-index")
        .expression_attribute_names("#id", "id")
        .expression_attribute_names("#bucket", "bucket")
        .expression_attribute_values(":id", AttributeValue::S(bot_id.to_owned()))
        .expression_attribute_values(":start_bucket", AttributeValue::S(start_bucket.to_string()))
        .expression_attribute_values(":end_bucket", AttributeValue::S(end_bucket.to_string()))
        
        // This sets the results to be sorted in descending order which we want
        .scan_index_forward(false)
        // Due to how the data is structured on the stats table we need to ALWAYS do a **between** condition between two time periods. 
        // It can be as simple as taking the current bucket "hour_{date}" and making {date} a _, which I think will work for our purposes. 
        .key_condition_expression("#id = :id and #bucket between :start_bucket and :end_bucket")
        .send().await.wrap_err("failed to get bot stats")?;
    
    // let output = client.query()
    //     .table_name(table_name)
    //     .index_name("period-time-index")
    //     .expression_attribute_names("#time", "time")
    //     .expression_attribute_names("#period", "period")
    //     .expression_attribute_values(":start", )
    
    if output.items.is_none() {
        bail!("no items were returned from dynamo query")
    }
    let items = output.items();
    
    println!("query returned {} items", items.len());
   
    for (_index, item) in items.iter().enumerate() {
        stats.push(match from_item(item.clone()) {
            Ok(a) => a,
            Err(e) => {
                let json: Value = from_item(item.clone()).unwrap();
                bail!("failed to deserialize: '{e}' \n {}", serde_json::to_string(&json).unwrap())
            },
        });
    }
    
    
    Ok(stats)
}

pub async fn get_queue_details(client: &Client, table_name: &str) -> color_eyre::Result<Vec<QueueDetails>> {
    let mut queues = vec![];
    
    let output = client
        .scan()
        .table_name(table_name)
        .return_consumed_capacity(ReturnConsumedCapacity::Total)
        .send()
        .await
        .wrap_err("failed to get queue stats")?;
    
    if output.items.is_none() {
        bail!("no items were returned from queue dynamo scan");
    }
    
    let items = output.items();
    
    for item in items.iter() {
        queues.push(
            match from_item(item.clone()) {
                Ok(a) => a,
                Err(e) => {
                    let json: Value = from_item(item.clone()).unwrap();
                    bail!("failed to deserialize: '{e}' \n {}", serde_json::to_string(&json).unwrap())
                },
            }
        )
    }
    
    Ok(queues)
    
    
}
