/*
 * Copyright (C) 2019 Romain Vimont
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

mod adb_monitor;
mod byte_buffer;

use crate::adb_monitor::{AdbMonitor, AdbMonitorCallback};
use std::env;
use std::process::{Command, Child};


struct AutoAdb {
    connect_command: Vec<String>,
    disconnect_command: Vec<Option<String>>,
	connected_process: Option<Child>, // 用于存储设备连接时执行的进程

}

impl AutoAdb {
    fn new(connect_command: Vec<String>, disconnect_command: Vec<Option<String>>) -> Self {
        Self {
            connect_command,
            disconnect_command,
			connected_process: None,

        }
    }
}

impl AdbMonitorCallback for AutoAdb {
    fn on_new_device_connected(&mut self, serial: &str) {
        let cmd = self
            .connect_command
            .iter()
            .map(|value| {
                // replace any {} parameter by the actual serial
                if "{}" == value {
                    serial.to_string()
                } else {
                    value.to_string()
                }
            })
            .collect::<Vec<_>>();
        println!("Detected device {}", serial);
        if let Ok(process) = Command::new(&cmd[0]).args(cmd.iter().skip(1)).spawn() {
            self.connected_process = Some(process);
        } else {
            eprintln!("Could not execute {:?}", cmd);
        }
    }
	// 新添加的设备断开连接回调函数
    fn on_device_disconnected(&mut self, serial: &str) {
    println!("Device disconnected: {}", serial);
    
 // 停止设备连接时执行的程序
	if let Some(mut process) = self.connected_process.take() {
		if let Err(err) = process.kill() {
			eprintln!("Error stopping connected process: {}", err);
		}
	}
	
    if let Some(cmd) = &self.disconnect_command[0] {
        let cmd = cmd
            .split_whitespace()
            .map(|value| {
                // replace any {} parameter by the actual serial
                if "{}" == value {
                    serial.to_string()
                } else {
                    value.to_string()
                }
            })
            .collect::<Vec<_>>();
        let process = Command::new(&cmd[0]).args(&cmd[1..]).spawn();
        if let Err(err) = process {
            eprintln!("Could not execute {:?}: {}", cmd, err);
        }
    }
    }
}

fn main() {
    // 获取命令行参数
    let args: Vec<String> = env::args().collect();

    // 确保至少有2个参数：可执行文件名、连接时的命令,断开连接时的命令可选
    if args.len() < 2 {
        eprintln!("Usage: {} <connect_command> [disconnect_command]", args[0]);
        return;
    }

    let connect_command = vec![args[1].clone()]; // 第一个参数是连接时的命令，需要包装为 Vec<String>
    let disconnect_command = if args.len() > 2 {
        vec![Some(args[2].clone())] // 如果提供了第二个参数，则将其作为断开连接时的命令
    } else {
        vec![None] // 如果没有提供第二个参数，则断开连接时的命令为空
    };

    // 创建 AutoAdb 实例并传递命令
    let auto_adb = AutoAdb::new(connect_command, disconnect_command);
	
    let mut adb_monitor = AdbMonitor::new(Box::new(auto_adb));
    adb_monitor.monitor();
}
