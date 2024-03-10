extern crate sysinfo;
use hyprland::data::{Client, Clients};
use hyprland::event_listener::EventListenerMutable as EventListener;
use hyprland::event_listener::{WindowEventData, WindowOpenEvent};
use hyprland::shared::Address;
use hyprland::prelude::*;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;
use sysinfo::{Pid, System, ThreadKind};

enum Event {
    Schedule,
    Active(Option<WindowEventData>),
    Open(WindowOpenEvent),
    Close(Address),
}

fn get_children_pids(system: &System, parent_pid: u32) -> Vec<u32> {
    let mut child_pids = Vec::new();
    let mut task_pids = Vec::new();
    if let Some(tasks) = system
        .process(Pid::from_u32(parent_pid))
        .expect("Problem!!!")
        .tasks()
    {
        for task in tasks {
            task_pids.push(task.as_u32());
        }
    }
    // println!("Task PIDs {task_pids:#?}");
    for (pid, process) in system.processes() {
        if task_pids.contains(&pid.as_u32())
            || process.thread_kind().unwrap_or(ThreadKind::Userland) == ThreadKind::Kernel
        {
            continue;
        }
        let ppid = process.parent().unwrap_or(Pid::from(0)).as_u32();
        let name = process.name();
        if ppid == parent_pid {
            println!("Checking {name:#?}");
            println!("{pid:#?} is child of {parent_pid:#?}");
            child_pids.push(pid.as_u32());
            let mut grandchildren = get_children_pids(system, pid.as_u32());
            child_pids.append(&mut grandchildren);
        }
    }
    child_pids
}

fn timer_tick(tx: Sender<Event>) {
    loop {
        tx.send(Event::Schedule).unwrap();
        thread::sleep(Duration::from_secs(1));
    }
}

fn hypr_events(tx: Sender<Event>) {
    let tx1 = tx.clone();
    let tx2 = tx.clone();
    let tx3 = tx.clone();
    let mut event_listener = EventListener::new();
    event_listener.add_active_window_change_handler(move |data, state| {
        println!("New active: {data:#?} {state:#?}");
        tx1.send(Event::Active(data)).unwrap();
    });
    event_listener.add_window_open_handler(move |data, _| {
        println!("New window: {data:#?}");
        tx2.send(Event::Open(data)).unwrap();
    });
    event_listener.add_window_close_handler(move |data, _| {
        println!("Closed window: {data:#?}");
        tx3.send(Event::Close(data)).unwrap();
    });
    let _ = event_listener.start_listener();
}

fn main() -> hyprland::Result<()> {
    let (tx, rx) = mpsc::channel();
    let tx1 = tx.clone();
    let tx2 = tx.clone();
    thread::spawn(move || timer_tick(tx1));
    thread::spawn(move || hypr_events(tx2));
    thread::spawn(move || {
        // TODO: new config object - reading from XDG_CONFIG_HOME/hyprland/hyprsuspender.toml
        // TODO: new suspender object - ctor creates map of Address->[class, pid, setting]
        // TODO: 3x new files: 1 for config.toml, 1 for process stuff and 1 for suspend logic

        // let clients = Clients::get()?.to_vec();
        // println!("{clients:#?}");
        //
        // let active_window = Client::get_active()?;
        // println!("{active_window:#?}");

        let mut system = System::new_all();
        system.refresh_all();

        let children = get_children_pids(&system, 2816);
        println!("Children PIDs: {:?}", children);

        for received in rx {
            match received {
                Event::Schedule => {
                    println!("Received schedule event.");
                }
                Event::Active(data) => {
                    println!("Received Active event: {data:#?}");
                }
                Event::Open(data) => {
                    println!("Received Open event: {data:#?}");
                }
                Event::Close(data) => {
                    println!("Received Close event: {data:#?}");
                }
            }
        }
    })
    .join()
    .unwrap();

    Ok(())
}
