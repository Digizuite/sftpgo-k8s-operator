use crate::default;
use crate::reconciler::Error;
use crds::{
    AzureBlobStorageAccessTier as CrdAccessTier, AzureBlobStorageAuthorization,
    FileSystem as CrdFileSystem,
};
use sftpgo_client::filesystem::{
    FileSystem as ClientFileSystem, FileSystemConfigAzureBlobStorage,
    FileSystemConfigAzureBlobStorageAccessTier, FileSystemConfigAzureBlobStorageAuthorization,
    FileSystemOsConfig, FileSystemProvider, SftpgoSecret, SftpgoSecretStatus,
};

pub async fn calculate_file_system(filesystem: &CrdFileSystem) -> Result<ClientFileSystem, Error> {
    let fs = match filesystem {
        CrdFileSystem::Local {
            read_buffer_size,
            write_buffer_size,
        } => ClientFileSystem {
            provider: FileSystemProvider::LocalFilesystem,
            os_config: Some(FileSystemOsConfig {
                read_buffer_size: *read_buffer_size,
                write_buffer_size: *write_buffer_size,
            }),
            ..default()
        },
        CrdFileSystem::AzureBlobStorage(blob) => ClientFileSystem {
            provider: FileSystemProvider::AzureBlobStorage,
            az_blob_config: Some(FileSystemConfigAzureBlobStorage {
                auth: match &blob.authorization {
                    AzureBlobStorageAuthorization::SharedKey {
                        account_key,
                        account_name,
                        container,
                    } => FileSystemConfigAzureBlobStorageAuthorization::SharedKey {
                        account_name: account_name.clone(),
                        container: container.clone(),
                        account_key: SftpgoSecret {
                            status: SftpgoSecretStatus::Plain,
                            payload: account_key.clone(),
                            ..default()
                        },
                    },
                    AzureBlobStorageAuthorization::SharedAccessSignatureUrl(url) => {
                        FileSystemConfigAzureBlobStorageAuthorization::SharedAccessSignatureUrl {
                            sas_url: SftpgoSecret {
                                status: SftpgoSecretStatus::Plain,
                                payload: url.clone(),
                                ..default()
                            },
                        }
                    }
                },
                endpoint: blob.endpoint.clone(),
                upload_part_size: blob.upload_part_size.clone(),
                upload_concurrency: blob.upload_concurrency.clone(),
                download_part_size: blob.download_part_size.clone(),
                download_concurrency: blob.download_concurrency.clone(),
                access_tier: blob.access_tier.map(|t| match t {
                    CrdAccessTier::Hot => FileSystemConfigAzureBlobStorageAccessTier::Hot,
                    CrdAccessTier::Cool => FileSystemConfigAzureBlobStorageAccessTier::Cool,
                    CrdAccessTier::Archive => FileSystemConfigAzureBlobStorageAccessTier::Archive,
                }),
                key_prefix: blob.key_prefix.clone(),
                use_emulator: blob.use_emulator,
            }),
            ..default()
        },
    };

    Ok(fs)
}
