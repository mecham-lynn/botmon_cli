use std::env::current_dir;
use std::{collections::HashMap, time::Instant};
use std::fs::read_to_string;

use aws_config::SdkConfig;
use aws_sdk_dynamodb::Client;
use chrono::{Duration, Utc};
use color_eyre::eyre::{bail, Context, Error};
use crossterm::event::{self, poll, Event, KeyCode, KeyEvent, KeyEventKind};
use futures::FutureExt;
use ratatui::Frame;
use throbber_widgets_tui::ThrobberState;
use tui_input::backend::crossterm::EventHandler;

use crate::dynamo::{get_all_bot_details, get_all_bot_stats_for_period, AllBucketsBuilder, Period};
use crate::pages::bus_select::BusSelectState;
use crate::{leo_config::LeoConfig, pages::bot::BotPageState, ui::render_ui, Tui, AppParams};

#[derive(Debug)]
pub struct AppState {
    pub mode: AppTab,
    pub bus_select: BusSelectState,
    pub tab_index: usize,
    pub chart_data: Vec<(f64, f64)>,
    pub bot_page: BotPageState,
    pub start_time: Instant,
    pub refresh_at: Instant,
    pub refresh_rate: Duration,
    pub selected_bus: Option<String>,
    pub buses: HashMap<String, LeoConfig>,
    pub loaded_config: Option<LeoConfig>,
    pub aws_config: SdkConfig,
    pub client: Client,
    pub throbber_state: ThrobberState,
    pub tick_rate: Duration,
    exit: bool
}

impl AppState {
    fn on_tick(&mut self) {
        self.throbber_state.calc_next()
    }
    /// Main loop for the application
    pub async fn run(&mut self, terminal: &mut Tui) -> color_eyre::Result<()> {
        let mut last_tick = Instant::now();
        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame))?;
            let timeout = self.tick_rate.to_std().unwrap().checked_sub(last_tick.elapsed()).unwrap_or_else(||Duration::seconds(0).to_std().unwrap());
            
            if poll(timeout)? {
                self.handle_events().await.wrap_err("handle events failed")?;

            }

            if last_tick.elapsed() >= self.tick_rate.to_std().unwrap() && self.mode == AppTab::Loading {
                self.on_tick();
                last_tick = Instant::now()
            }
            
        }
        Ok(())
    }
    
    fn render_frame(&mut self, frame: &mut Frame) {
        render_ui(frame, self)
        // frame.render_widget(self, frame.size());
    }
    
    async fn handle_events(&mut self) -> color_eyre::Result<()> {
        match event::read()? {
            
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => self
                .handle_key_event(key_event)
                .await
                .wrap_err_with(|| format!("handling key event failed: \n{key_event:#?}")),
            _ => Ok(()),
        }
    }
    
    fn return_home(&mut self) {
        self.tab_index = 0;
        self.mode = AppTab::Main
    }
    
    async fn handle_key_event(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        
        
        match key_event.code {
            KeyCode::Char('q') | KeyCode::Esc => Ok(self.exit()),
            KeyCode::Home => Ok(self.return_home()),
            // KeyCode::Left => self.decrement_count()?,
            // KeyCode::Right => self.increment_count()?,
            _ => {
                
                
                match self.mode {
                    AppTab::Loading => {
                        if self.bot_page.all_bots.as_ref().is_some_and(|a|!a.is_empty()) && !self.bot_page.stats.is_empty() {
                            self.mode = AppTab::Main;
                        }
                        Ok(())
                    }
                    AppTab::Main => self.navigate(key_event),
                    AppTab::Bot => {
                        let list_len =  if self.bot_page.search_results.is_empty() {
                            0_usize
                        } else {
                            self.bot_page.search_results.len()
                        };
                        
                        match key_event.code {
                            KeyCode::Down => {
                                if list_len > 0 {
                                    let index  = self.bot_page.current_select_index + list_len;
                                    self.bot_page.current_select_index = index.saturating_add(1) % list_len;
                                }
                            },
                            KeyCode::Up => {
                                if list_len > 0 {
                                    let index  = self.bot_page.current_select_index + list_len;
                                    self.bot_page.current_select_index = index.saturating_sub(1) % list_len;
                                }
                            }, 
                            KeyCode::Enter => {
                                if list_len > 0 {
                                    self.bot_page.selected_bot_name = Some(self.bot_page.search_results[self.bot_page.current_select_index].clone());
                                    self.bot_page.search.reset();
                                    self.bot_page.search_results.clear();
                                    self.bot_page.get_bot_details()?;
                                    self.mode = AppTab::BotView;
                                }
                            }
                            _ => {
                                // Do the search
                                self.bot_page.search.handle_event(&Event::Key(key_event));
                                self.bot_page.search_bots()
                            }
                        }
                        Ok(())
                    },
                    AppTab::BusSelect => {
                        let bus_len = if self.bus_select.buses.is_empty() {
                            0_usize
                        } else {
                            self.bus_select.buses.len()
                        };

                        match key_event.code {
                            KeyCode::Down => {
                                if bus_len > 0 {
                                    let index = self.bus_select.bus_selected_index + bus_len;
                                    self.bus_select.bus_selected_index = index.saturating_add(1) % bus_len;
                                    self.bus_select.vertical_scroll = self.bus_select.vertical_scroll.saturating_add(1);
                                    self.bus_select.vertical_scroll_state = self.bus_select.vertical_scroll_state.position(self.bus_select.vertical_scroll);
                                }
                            },
                            KeyCode::Up => {
                                if bus_len > 0 {
                                    let index  = self.bus_select.bus_selected_index + bus_len;
                                    self.bus_select.bus_selected_index = index.saturating_sub(1) % bus_len;
                                    self.bus_select.vertical_scroll = self.bus_select.vertical_scroll.saturating_sub(1);
                                    self.bus_select.vertical_scroll_state = self.bus_select.vertical_scroll_state.position(self.bus_select.vertical_scroll);
                                
                                }
                            }, 

                            KeyCode::Enter => {
                                if bus_len > 0 {
                                    self.selected_bus = Some(self.bus_select.buses[self.bus_select.bus_selected_index].clone());
                                    if let Some(selected_bus) = self.selected_bus.as_ref() {
                                        self.loaded_config = self.buses.get(selected_bus).cloned();
                                        //TODO: now we grab the data from db
                                        self.load_bot_data().await?;
                                        self.mode = AppTab::Main;
                                        // self.mode = AppTab::Loading;
                                        // let client = self.client.clone();
                                        // let stats_table = self.loaded_config.as_ref().unwrap().leo_stats.clone();
                                        // let cron_table = self.loaded_config.as_ref().unwrap().leo_cron.clone();
                                        // let bucket = AllBucketsBuilder::new(Period::Minute15)
                                        //         .past_ms(Duration::days(1))
                                        //         .build();
                                        // let mut tasks = vec![];
                                        
                                        
                                        // tasks.push(tokio::spawn(async move {
                                            
                                        //     get_all_bot_stats_for_period(client, stats_table, bucket).await
                                        // }).await?);
                                        // tasks.push(tokio::sp)
                                    }
                                }
                            }
                            a => {
                                bail!("invalid key {a:?} pressed");
                            }
                        }
                        Ok(())
                    }
                    AppTab::Queue => todo!(),
                    AppTab::BotView => match &mut self.bot_page.selected_bot {
                        Some(bot_view_state) => {
                            match key_event.code {
                                KeyCode::Up => {
                                    bot_view_state.vertical_scroll = bot_view_state.vertical_scroll.saturating_sub(1);
                                    bot_view_state.vertical_scroll_state = bot_view_state.vertical_scroll_state.position(bot_view_state.vertical_scroll);
                                },
                                KeyCode::Down => {
                                    bot_view_state.vertical_scroll = bot_view_state.vertical_scroll.saturating_add(1);
                                    bot_view_state.vertical_scroll_state = bot_view_state.vertical_scroll_state.position(bot_view_state.vertical_scroll);
                                },
                                KeyCode::Tab => {
                                    self.mode = AppTab::Bot
                                }
                                a => {
                                    bail!("invalid key {a:?} pressed");
                                }
                            }
                            Ok(())
                        },
                        None => bail!("cannot navigate a non-existant bot; \n{self:#?}"),
                    },
                }
            }
        }
    }
    
    fn exit(&mut self) {
        self.exit = true
    }
    
    
    
    pub fn gen_test_data(&mut self) {
        let mut data = vec![];
        let past_time = Utc::now() - Duration::minutes(200);
        for i in 0..200 {
            let num_executions = fastrand::u32(1..=3000) as f64;
            if i > 0 {
                data.push(((past_time + Duration::minutes(i as i64)).timestamp() as f64, num_executions))
            } else {
                data.push((past_time.timestamp() as f64, num_executions))
            }
        }
        
        self.chart_data = data
    
    }
    pub fn get_chart_data(&self) -> &[(f64, f64)] {
        &self.chart_data
    }

    async fn load_bot_data(&mut self) -> color_eyre::Result<()> {
   
        // Get the bot stats
        let bucket = AllBucketsBuilder::new(Period::Minute15)
            .past_ms(Duration::days(1))
            .build();

        let bots = get_all_bot_stats_for_period(&self.client, &self.loaded_config.as_ref().unwrap().leo_stats, bucket).await?;
        self.bot_page.stats = bots;

        let bot_settings = get_all_bot_details(&self.client, &self.loaded_config.as_ref().unwrap().leo_cron).await?;
        self.bot_page.all_bots = Some(bot_settings);

        self.bot_page.bot_names();

        Ok(())
    }

    pub async fn new(params: &AppParams) -> color_eyre::Result<Self> {
        let refresh_rate = params.refresh_time;
        let refresh_at = Instant::now() + refresh_rate.to_std()?;
        
        let buses = match &params.config_path {
            Some(path) => {
                let leo_string = read_to_string(path)?;
                let config: HashMap<String, LeoConfig> = serde_json::from_str(&leo_string)?;
                config
            },
            None => {
                let leo_string = read_to_string("./config.json").wrap_err_with(||current_dir().unwrap().to_str().unwrap().to_string())?;
                let config: HashMap<String, LeoConfig> = serde_json::from_str(&leo_string)?;
                config
            }
        };
        
        let selected_bus;
        let loaded_bus = match &params.bus {
            Some(bus) => {
                selected_bus = Some(bus.clone());
                match buses.get(bus) {
                    Some(value) => Some(value.clone()),
                    None => bail!("unable to find {bus} in leo config")
                }
            },
            None => {
                selected_bus = None;
                None
            }
        };
        let config: aws_config::SdkConfig = aws_config::load_from_env().await;
        let client = Client::new(&config);

        let mode = if loaded_bus.is_some() {
            AppTab::Main
        } else {
            AppTab::BusSelect
        };
        

        Ok(Self {
            start_time: Instant::now(),
            mode,
            tab_index: 0,
            chart_data: vec![],
            bot_page: BotPageState::default(),
            refresh_at,
            refresh_rate,
            exit: false,
            bus_select: BusSelectState::new(&buses),
            buses,
            selected_bus,
            loaded_config: loaded_bus,
            aws_config: config,
            client,
            throbber_state: ThrobberState::default(),
            tick_rate: Duration::milliseconds(250),
        })
    }
    
}

impl Navigate for AppState {
    fn navigate(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        const TAB_SIZE: usize = 2;
        match key_event.code {
            KeyCode::Down => {
                let index = self.tab_index + TAB_SIZE;
                self.tab_index = index.saturating_add(1) % TAB_SIZE;
            }
            KeyCode::Up => {
                let index = self.tab_index + TAB_SIZE;
                self.tab_index = index.saturating_sub(1) % TAB_SIZE;
            }
            KeyCode::Enter => self.mode = self.tab_index.into(),
            a => bail!("invalid key_code : {a:?}"),
        }
        Ok(())
    }
}


/// Control's which page that will show
#[derive(Debug, PartialEq)]
pub enum AppTab {
    Main,
    BusSelect,
    Bot,
    Queue,
    BotView,
    Loading,
}

impl AppTab {
    pub fn get_keys(&self) -> Vec<(&str, &str)>{
        let mut keys = vec![
            ("Home", "Main Menu"), 
            ("Esc|Q", "Quit")
        ];
        
        match self {
            AppTab::Main | AppTab::Bot | AppTab::Queue | AppTab::BusSelect => keys.append(&mut vec![
                ("↑", "Up"),
                ("↓", "Down"),
                ("Enter", "Select"),
                // ("Home", "Main Menu"),
                // ("Esc", "Quit")
            ]),
            AppTab::BotView => keys.append(&mut vec![
                ("↑", "Scroll Up"),
                ("↓", "Scroll Down"),
                ("Tab", "Back")
                // ("Home", "Main Menu"),
                // ("Esc", "Quit")
            ]),
            AppTab::Loading => {}
        }
        
        return keys;
    }
}

impl From<usize> for AppTab {
    fn from(value: usize) -> Self {
        if value == 0 {
            Self::Bot
        } else if value == 1 {
            Self::Queue
        } else {
            Self::Main
        }
    }
}

impl Default for AppTab {
    fn default() -> Self {
        Self::Main
    }
}

pub trait Page {
    fn get_keys(&self) -> Vec<(&str, &str)>;
}

pub trait Navigate {
    fn navigate(&mut self, key_event: KeyEvent) -> color_eyre::Result<()>;
}