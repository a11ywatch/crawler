/*
 * Copyright (c) A11yWatch, LLC. and its affiliates.
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 **/

use num_cpus;
use rocket;
use sysinfo::SystemExt;

#[get("/cpu")]
pub fn get_cpu() -> String {
	format!("Number of logic cores is {}", num_cpus::get())
}

#[get("/server-load")]
pub fn get_server_load() -> String {
	let mut system = sysinfo::System::new();

	system.refresh_all();

	let memory = format!("total memory: kB {}\n", system.get_total_memory());
	let used_memory = format!("used memory: kB {}\n", system.get_used_memory());
	let swap = format!("total swap: kB {}\n", system.get_total_swap());
	let used_swap = format!("used swap: kB {}\n", system.get_used_swap());

	format!("{}{}{}{}", memory, used_memory, swap, used_swap)
}
