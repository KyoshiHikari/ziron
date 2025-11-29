//! Job control system

use std::sync::{Arc, Mutex};

/// Job information
#[derive(Debug, Clone)]
pub struct Job {
    pub id: usize,
    pub command: String,
    pub pid: u32,
    pub status: JobStatus,
}

/// Job status
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JobStatus {
    Running,
    Stopped,
    Done(Option<i32>),
}

/// Job manager
#[derive(Clone)]
pub struct JobManager {
    jobs: Arc<Mutex<Vec<Job>>>,
    next_id: Arc<Mutex<usize>>,
}

impl JobManager {
    pub fn new() -> Self {
        Self {
            jobs: Arc::new(Mutex::new(Vec::new())),
            next_id: Arc::new(Mutex::new(1)),
        }
    }

    pub fn add_job(&self, command: String, pid: u32) -> usize {
        let mut jobs = self.jobs.lock().unwrap();
        let mut next_id = self.next_id.lock().unwrap();
        let id = *next_id;
        *next_id += 1;

        jobs.push(Job {
            id,
            command,
            pid,
            status: JobStatus::Running,
        });

        id
    }

    pub fn list_jobs(&self) -> Vec<Job> {
        let jobs = self.jobs.lock().unwrap();
        jobs.clone()
    }

    pub fn get_job(&self, spec: &str) -> Option<Job> {
        let jobs = self.jobs.lock().unwrap();
        
        match spec {
            "+" | "%+" => jobs.iter().find(|j| j.status == JobStatus::Running).cloned(),
            "-" | "%-" => {
                let running: Vec<_> = jobs.iter().filter(|j| j.status == JobStatus::Running).collect();
                if running.len() > 1 {
                    running.get(running.len() - 2).cloned().cloned()
                } else {
                    None
                }
            }
            _ => {
                if let Ok(id) = spec.trim_start_matches('%').parse::<usize>() {
                    jobs.iter().find(|j| j.id == id).cloned()
                } else {
                    None
                }
            }
        }
    }

    pub fn update_job_status(&self, pid: u32, status: JobStatus) {
        let mut jobs = self.jobs.lock().unwrap();
        if let Some(job) = jobs.iter_mut().find(|j| j.pid == pid) {
            job.status = status;
        }
    }

    pub fn remove_job(&self, id: usize) {
        let mut jobs = self.jobs.lock().unwrap();
        jobs.retain(|j| j.id != id);
    }
}

impl Default for JobManager {
    fn default() -> Self {
        Self::new()
    }
}

