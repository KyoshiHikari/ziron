//! Kubernetes module for Ziron

use ziron_core::error::Result;
use ziron_core::module::{ModuleContext, ModuleData};
use std::process::Command;

/// Kubernetes module implementation
pub struct KubernetesModule;

impl KubernetesModule {
    /// Fetch Kubernetes context information
    pub fn fetch_data(_context: &ModuleContext) -> Result<ModuleData> {
        let k8s_info = Self::get_k8s_info()?;

        if let Some(info) = k8s_info {
            let mut parts = vec![];
            
            if let Some(ref ctx) = info.context {
                parts.push(ctx.clone());
            }
            
            if let Some(ref namespace) = info.namespace {
                if namespace != "default" {
                    parts.push(format!("ns:{}", namespace));
                }
            }
            
            if let Some(ref cluster) = info.cluster {
                parts.push(format!("cluster:{}", cluster));
            }

            Ok(ModuleData {
                module: "kubernetes".to_string(),
                data: serde_json::json!({
                    "text": parts.join(" "),
                    "context": info.context,
                    "namespace": info.namespace,
                    "cluster": info.cluster,
                }),
                cached: false,
            })
        } else {
            Ok(ModuleData {
                module: "kubernetes".to_string(),
                data: serde_json::json!({
                    "text": "",
                    "context": null,
                }),
                cached: false,
            })
        }
    }

    fn get_k8s_info() -> Result<Option<K8sInfo>> {
        // Get kubeconfig path
        let kubeconfig = std::env::var("KUBECONFIG")
            .ok()
            .or_else(|| {
                std::env::var("HOME")
                    .ok()
                    .map(|home| format!("{}/.kube/config", home))
            });

        if kubeconfig.is_none() {
            return Ok(None);
        }

        // Get current context
        let context_output = Command::new("kubectl")
            .args(&["config", "current-context"])
            .output();

        let context = context_output
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    let ctx = String::from_utf8_lossy(&o.stdout).trim().to_string();
                    if !ctx.is_empty() {
                        Some(ctx)
                    } else {
                        None
                    }
                } else {
                    None
                }
            });

        if context.is_none() {
            return Ok(None);
        }

        // Get namespace
        let namespace_output = Command::new("kubectl")
            .args(&["config", "view", "--minify", "-o", "jsonpath={..namespace}"])
            .output();

        let namespace = namespace_output
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    let ns = String::from_utf8_lossy(&o.stdout).trim().to_string();
                    if !ns.is_empty() {
                        Some(ns)
                    } else {
                        Some("default".to_string())
                    }
                } else {
                    None
                }
            });

        // Get cluster name
        let cluster_output = Command::new("kubectl")
            .args(&["config", "view", "--minify", "-o", "jsonpath={.clusters[0].name}"])
            .output();

        let cluster = cluster_output
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    let cl = String::from_utf8_lossy(&o.stdout).trim().to_string();
                    if !cl.is_empty() {
                        Some(cl)
                    } else {
                        None
                    }
                } else {
                    None
                }
            });

        Ok(Some(K8sInfo {
            context,
            namespace,
            cluster,
        }))
    }
}

#[derive(Debug, Clone)]
struct K8sInfo {
    context: Option<String>,
    namespace: Option<String>,
    cluster: Option<String>,
}

