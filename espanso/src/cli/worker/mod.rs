/*
 * This file is part of espanso.
 *
 * Copyright (C) 2019-2021 Federico Terzi
 *
 * espanso is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * espanso is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with espanso.  If not, see <https://www.gnu.org/licenses/>.
 */

use funnel::Source;
use process::Matcher;
use ui::selector::MatchSelectorAdapter;

use crate::engine::{Engine, funnel, process, dispatch};
use super::{CliModule, CliModuleArgs};

mod ui;
mod config;
mod source;
mod matcher;
mod executor;

pub fn new() -> CliModule {
  #[allow(clippy::needless_update)]
  CliModule {
    requires_paths: true,
    requires_config: true,
    enable_logs: true,
    subcommand: "worker".to_string(),
    entry: worker_main,
    ..Default::default()
  }
}

fn worker_main(args: CliModuleArgs) {
  let config_store = args.config_store.expect("missing config store in worker main");
  let match_store = args.match_store.expect("missing match store in worker main");

  let app_info_provider = espanso_info::get_provider().expect("unable to initialize app info provider");
  let config_manager = config::ConfigManager::new(&*config_store, &*match_store, &*app_info_provider);
  let match_converter = matcher::convert::MatchConverter::new(&*config_store, &*match_store);

  let detect_source = source::detect::init_and_spawn().expect("failed to initialize detector module");
  let sources: Vec<&dyn Source> = vec![&detect_source];
  let funnel = funnel::default(&sources);

  let matcher = matcher::rolling::RollingMatcherAdapter::new(&match_converter.get_rolling_matches());
  let matchers: Vec<&dyn Matcher<matcher::MatcherState>> = vec![&matcher];
  let selector = MatchSelectorAdapter::new();
  let mut processor = process::default(&matchers, &config_manager, &selector);

  let text_injector = executor::text_injector::TextInjectorAdapter::new();
  let dispatcher = dispatch::default(&text_injector);

  let mut engine = Engine::new(&funnel, &mut processor, &dispatcher);
  engine.run();
}
