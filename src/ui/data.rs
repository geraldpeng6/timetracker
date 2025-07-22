use crate::core::tracker::{ActivityRecord, TimeTracker};
use crate::ui::components::{
    AppTableItem, ProductivityCategory, RecentActivityItem, TimeRangeFilter, UnifiedActivityItem,
    WindowItem,
};

/// 数据管理器
pub struct DataManager {
    tracker: TimeTracker,
}

impl DataManager {
    pub fn new(data_file: String) -> anyhow::Result<Self> {
        let tracker = TimeTracker::new(data_file, 1); // 改为1秒间隔，实现实时监控
                                                      // 延迟加载数据，避免在TUI初始化时阻塞
        Ok(Self { tracker })
    }

    /// 延迟初始化数据（在需要时调用）
    pub fn initialize_data(&mut self) -> anyhow::Result<()> {
        self.tracker.load_data()
    }

    /// 获取统一活动数据 - 合并应用和窗口信息
    pub fn get_unified_activities(&self, time_filter: TimeRangeFilter) -> Vec<UnifiedActivityItem> {
        // 如果数据未初始化，返回空数据
        if self.tracker.data.activities.is_empty() && self.tracker.data.current_activity.is_none() {
            return Vec::new();
        }

        let activities = self.get_filtered_activities(time_filter);
        let mut activity_map: std::collections::HashMap<
            String,
            (
                u64,                           // total_duration
                u64,                           // recent_duration (最近一次的时长)
                usize,                         // activity_count
                chrono::DateTime<chrono::Utc>, // last_active
                chrono::DateTime<chrono::Utc>, // first_active
            ),
        > = std::collections::HashMap::new();

        // 获取当前活动的应用和窗口（如果有的话）
        let current_activity = self.tracker.data.current_activity.as_ref();
        let current_key = current_activity.map(|a| format!("{} - {}", a.app_name, a.window_title));

        // 处理已完成的活动
        for activity in activities {
            let key = format!("{} - {}", activity.app_name, activity.window_title);
            let entry = activity_map.entry(key).or_insert((
                0,
                0,
                0,
                activity.start_time,
                activity.start_time,
            ));

            entry.0 += activity.duration; // total_duration
            entry.2 += 1; // activity_count

            // 更新最近一次的时长（最新的活动记录）
            if activity.start_time >= entry.3 {
                entry.1 = activity.duration; // recent_duration
                entry.3 = activity.start_time; // last_active
            }

            // 更新首次活动时间
            if activity.start_time < entry.4 {
                entry.4 = activity.start_time; // first_active
            }
        }

        // 如果有当前活动，也加入统计
        if let Some(current) = current_activity {
            let key = format!("{} - {}", current.app_name, current.window_title);
            let current_duration = current.duration; // 使用已经计算好的duration

            let entry = activity_map.entry(key).or_insert((
                0,
                0,
                0,
                current.start_time,
                current.start_time,
            ));

            entry.0 += current_duration; // total_duration
            entry.2 += 1; // activity_count

            // 当前活动总是最新的，所以设置为recent_duration
            entry.1 = current_duration; // recent_duration
            entry.3 = chrono::Utc::now(); // last_active

            // 更新首次活动时间
            if current.start_time < entry.4 {
                entry.4 = current.start_time; // first_active
            }
        }

        let mut items: Vec<UnifiedActivityItem> = activity_map
            .into_iter()
            .map(
                |(
                    key,
                    (total_duration, recent_duration, activity_count, last_active, first_active),
                )| {
                    let parts: Vec<&str> = key.splitn(2, " - ").collect();
                    let app_name = parts[0].to_string();
                    let window_title = parts.get(1).unwrap_or(&"").to_string();

                    let is_currently_active = current_key.as_ref().map_or(false, |ck| ck == &key);
                    let productivity_category = self.categorize_app_productivity(&app_name);

                    UnifiedActivityItem {
                        app_name,
                        window_title,
                        total_duration,
                        recent_duration,
                        activity_count,
                        last_active,
                        first_active,
                        is_currently_active,
                        productivity_category,
                    }
                },
            )
            .collect();

        // 按总时长降序排序
        items.sort_by(|a, b| b.total_duration.cmp(&a.total_duration));
        items
    }

    /// 获取应用程序表格数据
    pub fn get_app_table_data(&self, time_filter: TimeRangeFilter) -> Vec<AppTableItem> {
        // 如果数据未初始化，返回空数据
        if self.tracker.data.activities.is_empty() && self.tracker.current_activity.is_none() {
            return Vec::new();
        }

        let activities = self.get_filtered_activities(time_filter);
        let mut app_map: std::collections::HashMap<String, (u64, Vec<&ActivityRecord>)> =
            std::collections::HashMap::new();

        for activity in activities {
            let entry = app_map
                .entry(activity.app_name.clone())
                .or_insert((0, Vec::new()));
            entry.0 += activity.duration;
            entry.1.push(activity);
        }

        let mut items: Vec<AppTableItem> = app_map
            .into_iter()
            .map(|(app_name, (total_duration, activities))| {
                // 创建窗口项
                let mut window_map: std::collections::HashMap<
                    String,
                    (u64, usize, chrono::DateTime<chrono::Utc>),
                > = std::collections::HashMap::new();

                for activity in &activities {
                    let entry = window_map.entry(activity.window_title.clone()).or_insert((
                        0,
                        0,
                        activity.start_time,
                    ));
                    entry.0 += activity.duration;
                    entry.1 += 1;
                    if activity.start_time > entry.2 {
                        entry.2 = activity.start_time;
                    }
                }

                let windows: Vec<WindowItem> = window_map
                    .into_iter()
                    .map(
                        |(window_title, (duration, activity_count, last_active))| WindowItem {
                            window_title,
                            duration,
                            last_active,
                            activity_count,
                        },
                    )
                    .collect();

                let last_active = activities
                    .iter()
                    .map(|a| a.start_time)
                    .max()
                    .unwrap_or_else(chrono::Utc::now);

                AppTableItem {
                    app_name,
                    total_duration,
                    window_count: windows.len(),
                    windows,
                    is_expanded: false,
                    last_active,
                }
            })
            .collect();

        items.sort_by(|a, b| b.total_duration.cmp(&a.total_duration));
        items
    }

    /// 获取窗口数据
    pub fn get_window_data(&self, time_filter: TimeRangeFilter) -> Vec<WindowItem> {
        let activities = self.get_filtered_activities(time_filter);
        let mut window_map: std::collections::HashMap<
            String,
            (u64, usize, chrono::DateTime<chrono::Utc>),
        > = std::collections::HashMap::new();

        for activity in activities {
            let entry = window_map.entry(activity.window_title.clone()).or_insert((
                0,
                0,
                activity.start_time,
            ));
            entry.0 += activity.duration;
            entry.1 += 1;
            if activity.start_time > entry.2 {
                entry.2 = activity.start_time;
            }
        }

        let mut items: Vec<WindowItem> = window_map
            .into_iter()
            .map(
                |(window_title, (duration, activity_count, last_active))| WindowItem {
                    window_title,
                    duration,
                    last_active,
                    activity_count,
                },
            )
            .collect();

        items.sort_by(|a, b| b.duration.cmp(&a.duration));
        items
    }

    /// 获取最近活动数据
    pub fn get_recent_activities(&self, limit: usize) -> Vec<RecentActivityItem> {
        let activities = self.tracker.get_recent_activities(limit);
        activities
            .into_iter()
            .map(|activity| RecentActivityItem {
                app_name: activity.app_name.clone(),
                window_title: activity.window_title.clone(),
                start_time: activity.start_time,
                end_time: activity.end_time,
                duration: activity.duration,
            })
            .collect()
    }

    /// 删除指定的活动记录
    pub fn delete_activity(&mut self, index: usize) -> anyhow::Result<bool> {
        if index < self.tracker.data.activities.len() {
            let removed_activity = self.tracker.data.activities.remove(index);
            log::info!(
                "删除活动记录: {} - {} ({}秒)",
                removed_activity.app_name,
                removed_activity.window_title,
                removed_activity.duration
            );

            // 保存数据到文件
            self.tracker.save_data()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// 批量删除活动记录
    pub fn delete_activities(&mut self, indices: Vec<usize>) -> anyhow::Result<usize> {
        let mut sorted_indices = indices;
        sorted_indices.sort_by(|a, b| b.cmp(a)); // 从大到小排序，避免索引错位

        let mut deleted_count = 0;
        for index in sorted_indices {
            if index < self.tracker.data.activities.len() {
                let removed_activity = self.tracker.data.activities.remove(index);
                log::info!(
                    "批量删除活动记录: {} - {}",
                    removed_activity.app_name,
                    removed_activity.window_title
                );
                deleted_count += 1;
            }
        }

        if deleted_count > 0 {
            self.tracker.save_data()?;
        }

        Ok(deleted_count)
    }

    /// 删除指定应用的所有活动记录
    pub fn delete_app_activities(&mut self, app_name: &str) -> anyhow::Result<usize> {
        let initial_count = self.tracker.data.activities.len();
        self.tracker
            .data
            .activities
            .retain(|activity| activity.app_name != app_name);
        let deleted_count = initial_count - self.tracker.data.activities.len();

        if deleted_count > 0 {
            log::info!(
                "删除应用 {} 的所有活动记录，共 {} 条",
                app_name,
                deleted_count
            );
            self.tracker.save_data()?;
        }

        Ok(deleted_count)
    }

    /// 删除指定应用和窗口的最近一条活动记录
    pub fn delete_recent_activity_by_app_window(
        &mut self,
        app_name: &str,
        window_title: &str,
    ) -> anyhow::Result<bool> {
        // 找到匹配的最近一条记录（按开始时间倒序）
        let mut found_index = None;
        for (index, activity) in self.tracker.data.activities.iter().enumerate().rev() {
            if activity.app_name == app_name && activity.window_title == window_title {
                found_index = Some(index);
                break;
            }
        }

        if let Some(index) = found_index {
            let removed_activity = self.tracker.data.activities.remove(index);
            log::info!(
                "删除最近的活动记录: {} - {} ({}秒, 开始时间: {})",
                removed_activity.app_name,
                removed_activity.window_title,
                removed_activity.duration,
                removed_activity.start_time.format("%Y-%m-%d %H:%M:%S")
            );

            // 保存数据到文件
            self.tracker.save_data()?;
            Ok(true)
        } else {
            log::warn!("未找到匹配的活动记录: {} - {}", app_name, window_title);
            Ok(false)
        }
    }

    /// 删除指定时间范围内的活动记录
    pub fn delete_activities_in_range(
        &mut self,
        start_time: chrono::DateTime<chrono::Utc>,
        end_time: chrono::DateTime<chrono::Utc>,
    ) -> anyhow::Result<usize> {
        let initial_count = self.tracker.data.activities.len();
        self.tracker
            .data
            .activities
            .retain(|activity| activity.start_time < start_time || activity.start_time > end_time);
        let deleted_count = initial_count - self.tracker.data.activities.len();

        if deleted_count > 0 {
            log::info!(
                "删除时间范围 {} 到 {} 的活动记录，共 {} 条",
                start_time.format("%Y-%m-%d %H:%M:%S"),
                end_time.format("%Y-%m-%d %H:%M:%S"),
                deleted_count
            );
            self.tracker.save_data()?;
        }

        Ok(deleted_count)
    }

    /// 编辑活动记录
    pub fn edit_activity(
        &mut self,
        index: usize,
        new_app_name: Option<String>,
        new_window_title: Option<String>,
        new_duration: Option<u64>,
    ) -> anyhow::Result<bool> {
        if index >= self.tracker.data.activities.len() {
            return Ok(false);
        }

        let activity = &mut self.tracker.data.activities[index];
        let mut changed = false;

        if let Some(app_name) = new_app_name {
            if activity.app_name != app_name {
                log::info!("修改活动应用名称: {} -> {}", activity.app_name, app_name);
                activity.app_name = app_name;
                changed = true;
            }
        }

        if let Some(window_title) = new_window_title {
            if activity.window_title != window_title {
                log::info!(
                    "修改活动窗口标题: {} -> {}",
                    activity.window_title,
                    window_title
                );
                activity.window_title = window_title;
                changed = true;
            }
        }

        if let Some(duration) = new_duration {
            if activity.duration != duration {
                log::info!(
                    "修改活动持续时间: {}秒 -> {}秒",
                    activity.duration,
                    duration
                );
                activity.duration = duration;

                // 更新结束时间
                activity.end_time =
                    Some(activity.start_time + chrono::Duration::seconds(duration as i64));
                changed = true;
            }
        }

        if changed {
            self.tracker.save_data()?;
        }

        Ok(changed)
    }

    /// 合并相同应用和窗口的活动记录
    pub fn merge_similar_activities(
        &mut self,
        app_name: &str,
        window_title: &str,
    ) -> anyhow::Result<usize> {
        let mut activities_to_merge: Vec<usize> = Vec::new();

        // 找到所有匹配的活动记录
        for (index, activity) in self.tracker.data.activities.iter().enumerate() {
            if activity.app_name == app_name && activity.window_title == window_title {
                activities_to_merge.push(index);
            }
        }

        if activities_to_merge.len() <= 1 {
            return Ok(0); // 没有需要合并的记录
        }

        // 计算总持续时间和时间范围
        let mut total_duration = 0u64;
        let mut earliest_start = chrono::Utc::now();
        let mut latest_end = chrono::DateTime::<chrono::Utc>::from_timestamp(0, 0).unwrap();

        for &index in &activities_to_merge {
            let activity = &self.tracker.data.activities[index];
            total_duration += activity.duration;

            if activity.start_time < earliest_start {
                earliest_start = activity.start_time;
            }

            if let Some(end_time) = activity.end_time {
                if end_time > latest_end {
                    latest_end = end_time;
                }
            }
        }

        // 创建合并后的活动记录
        let merged_activity = ActivityRecord {
            app_name: app_name.to_string(),
            window_title: window_title.to_string(),
            start_time: earliest_start,
            end_time: Some(latest_end),
            duration: total_duration,
            process_id: self.tracker.data.activities[activities_to_merge[0]].process_id,
            app_path: self.tracker.data.activities[activities_to_merge[0]]
                .app_path
                .clone(),
            bundle_id: self.tracker.data.activities[activities_to_merge[0]]
                .bundle_id
                .clone(),
            window_geometry: None, // 合并活动不保留窗口几何信息
            confidence: 1.0,       // 合并活动的置信度设为1.0
        };

        // 删除原有记录（从后往前删除避免索引错位）
        let mut sorted_indices = activities_to_merge.clone();
        sorted_indices.sort_by(|a, b| b.cmp(a));
        for index in sorted_indices {
            self.tracker.data.activities.remove(index);
        }

        // 添加合并后的记录
        self.tracker.data.activities.push(merged_activity);

        let merged_count = activities_to_merge.len();
        log::info!(
            "合并 {} 条相同的活动记录: {} - {}",
            merged_count,
            app_name,
            window_title
        );

        self.tracker.save_data()?;
        Ok(merged_count)
    }

    /// 获取所有活动记录（用于编辑界面）
    pub fn get_all_activities(&self) -> &Vec<ActivityRecord> {
        &self.tracker.data.activities
    }

    /// 获取可变的活动记录引用（用于直接编辑）
    pub fn get_activity_mut(&mut self, index: usize) -> Option<&mut ActivityRecord> {
        self.tracker.data.activities.get_mut(index)
    }

    /// 保存数据到文件
    pub fn save_data(&self) -> anyhow::Result<()> {
        self.tracker.save_data()
    }

    /// 重新加载数据
    pub fn reload_data(&mut self) -> anyhow::Result<()> {
        self.tracker.load_data()
    }

    /// 根据时间过滤器获取活动数据
    fn get_filtered_activities(&self, time_filter: TimeRangeFilter) -> Vec<&ActivityRecord> {
        match time_filter {
            TimeRangeFilter::All => self.tracker.get_activities().iter().collect(),
            _ => {
                // 对于其他时间过滤器，我们需要实现时间过滤逻辑
                // 这里先返回所有活动，后续可以根据需要实现具体的时间过滤
                self.tracker.get_activities().iter().collect()
            }
        }
    }

    /// 应用分类（用于生产力评分）
    fn categorize_app(&self, app_name: &str) -> AppCategory {
        // 创建应用分类预设表
        let productive_apps = [
            // 开发工具
            "vscode",
            "visual studio code",
            "code",
            "xcode",
            "intellij",
            "idea",
            "pycharm",
            "webstorm",
            "android studio",
            "eclipse",
            "vim",
            "emacs",
            "neovim",
            "sublime text",
            "atom",
            "brackets",
            "phpstorm",
            "clion",
            "rider",
            // 终端和命令行工具
            "terminal",
            "iterm",
            "iterm2",
            "warp",
            "hyper",
            "alacritty",
            "kitty",
            "powershell",
            "cmd",
            "bash",
            "zsh",
            "fish",
            // 开发相关工具
            "git",
            "github desktop",
            "sourcetree",
            "tower",
            "fork",
            "gitkraken",
            "docker",
            "docker desktop",
            "kubernetes",
            "postman",
            "insomnia",
            "paw",
            "tableplus",
            "sequel pro",
            "dbeaver",
            "navicat",
            "mongodb compass",
            // 设计工具
            "figma",
            "sketch",
            "adobe xd",
            "photoshop",
            "illustrator",
            "indesign",
            "after effects",
            "premiere pro",
            "final cut pro",
            "davinci resolve",
            "blender",
            "maya",
            "3ds max",
            "cinema 4d",
            "zbrush",
            // 办公软件
            "microsoft word",
            "word",
            "microsoft excel",
            "excel",
            "microsoft powerpoint",
            "powerpoint",
            "microsoft outlook",
            "outlook",
            "microsoft teams",
            "teams",
            "onenote",
            "google docs",
            "google sheets",
            "google slides",
            "google drive",
            "pages",
            "numbers",
            "keynote",
            "libreoffice",
            "openoffice",
            // 笔记和文档
            "notion",
            "obsidian",
            "typora",
            "bear",
            "ulysses",
            "scrivener",
            "evernote",
            "onenote",
            "joplin",
            "logseq",
            "roam research",
            "markdown editor",
            "marktext",
            "zettlr",
            // 通讯和协作
            "slack",
            "microsoft teams",
            "zoom",
            "skype",
            "discord",
            "telegram",
            "whatsapp",
            "wechat",
            "dingtalk",
            "feishu",
            "lark",
            // 项目管理
            "jira",
            "trello",
            "asana",
            "monday",
            "clickup",
            "linear",
            "height",
            "todoist",
            "things",
            "omnifocus",
            "taskwarrior",
            "org-mode",
            // 浏览器（工作相关）
            "chrome",
            "firefox",
            "safari",
            "edge",
            "brave",
            "opera",
            "vivaldi",
            // 其他生产力工具
            "alfred",
            "raycast",
            "spotlight",
            "launcher",
            "quicksilver",
            "1password",
            "bitwarden",
            "lastpass",
            "keychain access",
            "calculator",
            "calendar",
            "contacts",
            "mail",
            "notes",
        ];

        let unproductive_apps = [
            // 视频娱乐
            "youtube",
            "netflix",
            "hulu",
            "disney+",
            "amazon prime",
            "hbo max",
            "twitch",
            "bilibili",
            "iqiyi",
            "youku",
            "tencent video",
            "douyin",
            "tiktok",
            "vlc",
            "quicktime",
            "mpv",
            "plex",
            "kodi",
            "infuse",
            // 音乐娱乐
            "spotify",
            "apple music",
            "youtube music",
            "soundcloud",
            "pandora",
            "tidal",
            "deezer",
            "qq music",
            "netease music",
            "xiami music",
            // 游戏
            "steam",
            "epic games",
            "origin",
            "uplay",
            "battle.net",
            "gog galaxy",
            "minecraft",
            "world of warcraft",
            "league of legends",
            "dota 2",
            "counter-strike",
            "valorant",
            "overwatch",
            "fortnite",
            "apex legends",
            "among us",
            "fall guys",
            "rocket league",
            "fifa",
            "nba 2k",
            "civilization",
            "age of empires",
            "starcraft",
            "diablo",
            "hearthstone",
            // 社交媒体
            "facebook",
            "instagram",
            "twitter",
            "snapchat",
            "linkedin",
            "pinterest",
            "reddit",
            "tumblr",
            "weibo",
            "zhihu",
            "xiaohongshu",
            "douban",
            // 购物
            "amazon",
            "ebay",
            "taobao",
            "tmall",
            "jd",
            "pinduoduo",
            "shopify",
            // 新闻和阅读（娱乐性）
            "news",
            "flipboard",
            "pocket",
            "instapaper",
            "feedly",
            "reeder",
            // 其他娱乐
            "photos",
            "preview",
            "image viewer",
            "comic reader",
            "manga reader",
            "podcast",
            "audiobook",
            "kindle",
            "ibooks",
            "goodreads",
        ];

        let neutral_apps = [
            // 系统工具
            "finder",
            "file explorer",
            "explorer",
            "nautilus",
            "dolphin",
            "thunar",
            "activity monitor",
            "task manager",
            "system monitor",
            "htop",
            "top",
            "system preferences",
            "settings",
            "control panel",
            "registry editor",
            // 网络工具
            "network utility",
            "wifi analyzer",
            "speedtest",
            "ping",
            "traceroute",
            // 文件管理
            "7-zip",
            "winrar",
            "the unarchiver",
            "keka",
            "betterzip",
            "archive utility",
            "dropbox",
            "google drive",
            "onedrive",
            "icloud",
            "box",
            "mega",
            // 系统维护
            "disk utility",
            "cleanmymac",
            "ccleaner",
            "malwarebytes",
            "antivirus",
            "backup",
            "time machine",
            "carbon copy cloner",
            "superduper",
            // 其他工具
            "pdf reader",
            "adobe reader",
            "preview",
            "skim",
            "foxit reader",
            "text editor",
            "notepad",
            "textedit",
            "gedit",
            "nano",
            "clock",
            "timer",
            "stopwatch",
            "weather",
            "maps",
            "gps",
        ];

        let app_lower = app_name.to_lowercase();

        // 检查生产力应用
        for app in &productive_apps {
            if app_lower.contains(app) || app.contains(&app_lower) {
                return AppCategory::Productive;
            }
        }

        // 检查娱乐应用
        for app in &unproductive_apps {
            if app_lower.contains(app) || app.contains(&app_lower) {
                return AppCategory::Unproductive;
            }
        }

        // 检查中性应用
        for app in &neutral_apps {
            if app_lower.contains(app) || app.contains(&app_lower) {
                return AppCategory::Neutral;
            }
        }

        // 默认为中性
        AppCategory::Neutral
    }

    /// 应用生产力分类（用于UI显示）
    fn categorize_app_productivity(&self, app_name: &str) -> ProductivityCategory {
        match self.categorize_app(app_name) {
            AppCategory::Productive => ProductivityCategory::Productive,
            AppCategory::Neutral => ProductivityCategory::Neutral,
            AppCategory::Unproductive => ProductivityCategory::Unproductive,
        }
    }

    /// 获取统一活动数据 - 合并应用和窗口信息（带排序）
    pub fn get_unified_activities_sorted(
        &self,
        time_filter: TimeRangeFilter,
        sort_by: crate::ui::components::SortBy,
        sort_order: crate::ui::components::SortOrder,
    ) -> Vec<UnifiedActivityItem> {
        let mut items = self.get_unified_activities(time_filter);

        // 使用稳定排序，避免相同值的项目闪动
        items.sort_by(|a, b| {
            use crate::ui::components::{SortBy, SortOrder};
            use std::cmp::Ordering;

            let primary_cmp = match sort_by {
                SortBy::Duration => b.total_duration.cmp(&a.total_duration),
                SortBy::AppName => a.app_name.cmp(&b.app_name),
                SortBy::WindowTitle => a.window_title.cmp(&b.window_title),
                SortBy::ActivityCount => b.activity_count.cmp(&a.activity_count),
                SortBy::StartTime => b.first_active.cmp(&a.first_active),
                SortBy::EndTime => b.last_active.cmp(&a.last_active),
            };

            // 如果主要排序字段相同，使用次要排序字段确保稳定性
            let secondary_cmp = match primary_cmp {
                Ordering::Equal => match sort_by {
                    SortBy::Duration => a.app_name.cmp(&b.app_name),
                    SortBy::AppName => b.total_duration.cmp(&a.total_duration),
                    SortBy::WindowTitle => b.total_duration.cmp(&a.total_duration),
                    SortBy::ActivityCount => a.app_name.cmp(&b.app_name),
                    SortBy::StartTime => a.app_name.cmp(&b.app_name),
                    SortBy::EndTime => a.app_name.cmp(&b.app_name),
                },
                other => other,
            };

            // 如果次要排序也相同，使用第三级排序确保完全稳定
            let tertiary_cmp = match secondary_cmp {
                Ordering::Equal => {
                    // 使用窗口标题作为最终的稳定排序字段
                    a.window_title.cmp(&b.window_title)
                }
                other => other,
            };

            // 根据排序顺序调整结果
            match sort_order {
                SortOrder::Ascending => tertiary_cmp,
                SortOrder::Descending => tertiary_cmp.reverse(),
            }
        });

        items
    }

    /// 刷新数据
    pub fn refresh(&mut self) -> anyhow::Result<()> {
        // 重新加载数据文件，以获取最新的活动记录
        self.tracker.load_data()?;
        Ok(())
    }

    /// 获取当前正在进行的活动
    pub fn get_current_activity(&self) -> Option<&ActivityRecord> {
        self.tracker.current_activity.as_ref()
    }
}

/// 应用分类枚举
#[derive(Debug, Clone, PartialEq)]
enum AppCategory {
    Productive,   // 生产力应用
    Neutral,      // 中性应用
    Unproductive, // 娱乐/非生产力应用
}
