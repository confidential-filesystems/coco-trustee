// Copyright (c) 2023 by Alibaba.
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

use super::{Repository, ResourceDesc};
use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use log::info;
use std::process::{Command, Stdio};
use std::io::{self, Read};
use attestation_service::cfs;

pub const DEFAULT_REPO_DIR_PATH: &str = "/opt/confidential-containers/kbs/repository";

#[derive(Debug, Deserialize, Clone)]
pub struct LocalFsRepoDesc {
    pub dir_path: Option<String>,
}

impl Default for LocalFsRepoDesc {
    fn default() -> Self {
        Self {
            dir_path: Some(DEFAULT_REPO_DIR_PATH.to_string()),
        }
    }
}

pub struct LocalFs {
    pub repo_dir_path: String,
}

#[async_trait::async_trait]
impl Repository for LocalFs {
    async fn read_secret_resource(&self, resource_desc: ResourceDesc) -> Result<Vec<u8>> {
        let mut resource_path = PathBuf::from(&self.repo_dir_path);

        let ref_resource_path = format!(
            "{}/{}/{}",
            resource_desc.repository_name, resource_desc.resource_type, resource_desc.resource_tag
        );
        info!("read resource {}", ref_resource_path);

        // only for test
        let cfsi = cfs::Cfs::new("".to_string())?;
        let set_res = cfsi.set_resource(resource_desc.repository_name.clone(),
                                        resource_desc.resource_type.clone(),
                                        resource_desc.resource_tag.clone(),
                                        "test-data-1".to_string())
            .await?;
        info!("confilesystem - cfsi.set_resource() -> set_res = {:?}", set_res);
        let get_res = cfsi.get_resource(resource_desc.repository_name.clone(),
                                        resource_desc.resource_type.clone(),
                                        resource_desc.resource_tag.clone())
            .await?;
        info!("confilesystem - cfsi.get_resource() -> get_res = {:?}", get_res);

        let mut output = Command::new("cfs-resource")
            .arg("get")
            .arg("-d")
            .arg(&self.repo_dir_path)
            .arg("-r").arg(resource_desc.repository_name)
            .arg("-k").arg(resource_desc.resource_type)
            .arg("-t").arg(resource_desc.resource_tag)
            .stdout(Stdio::piped())
            .spawn()?;
        let mut stdout = output.stdout.take().expect("Failed to take stdout");
        let mut buffer: Vec<u8> = Vec::new();
        stdout.read_to_end(&mut buffer)?;

        let status = output.wait()?;
        if !status.success() {
            return Err(io::Error::new(io::ErrorKind::Other,
                                      format!("fail to read {}", ref_resource_path),).into());
        }
        Ok(buffer)
        /*
        resource_path.push(ref_resource_path);

        let resource_byte = tokio::fs::read(&resource_path)
            .await
            .context("read resource from local fs")?;
        Ok(resource_byte)
         */
    }

    async fn write_secret_resource(
        &mut self,
        resource_desc: ResourceDesc,
        data: &[u8],
    ) -> Result<()> {
        let mut resource_path = PathBuf::from(&self.repo_dir_path);
        resource_path.push(resource_desc.repository_name);
        resource_path.push(resource_desc.resource_type);

        if !Path::new(&resource_path).exists() {
            tokio::fs::create_dir_all(&resource_path)
                .await
                .context("create new resource path")?;
        }

        resource_path.push(resource_desc.resource_tag);

        tokio::fs::write(resource_path, data)
            .await
            .context("write local fs")
    }
}

impl LocalFs {
    pub fn new(repo_desc: &LocalFsRepoDesc) -> Result<Self> {
        Ok(Self {
            repo_dir_path: repo_desc
                .dir_path
                .clone()
                .unwrap_or(DEFAULT_REPO_DIR_PATH.to_string()),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::resource::{
        local_fs::{LocalFs, LocalFsRepoDesc},
        Repository, ResourceDesc,
    };

    const TEST_DATA: &[u8] = b"testdata";

    #[tokio::test]
    async fn write_and_read_resource() {
        let tmp_dir = tempfile::tempdir().expect("create temp dir failed");
        let repo_desc = LocalFsRepoDesc {
            dir_path: Some(tmp_dir.path().to_string_lossy().to_string()),
        };

        let mut local_fs = LocalFs::new(&repo_desc).expect("create local fs failed");
        let resource_desc = ResourceDesc {
            repository_name: "default".into(),
            resource_type: "test".into(),
            resource_tag: "test".into(),
        };

        local_fs
            .write_secret_resource(resource_desc.clone(), TEST_DATA)
            .await
            .expect("write secret resource failed");
        let data = local_fs
            .read_secret_resource(resource_desc)
            .await
            .expect("read secret resource failed");

        assert_eq!(&data[..], TEST_DATA);
    }
}
