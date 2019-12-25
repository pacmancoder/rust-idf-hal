use idf_sys::{
    nvs::*,
    error::*,
};
use crate::peripherals::NvsPeripherals;
use crate::nvs::NvsError::IdfError;


#[non_exhaustive]
pub enum NvsError {
    InvalidPartitionId,
    PartitionCorrupted,
    PartitionNotFound,
    AlreadyInitialized,
    IdfError(esp_err_t),
}


const MAX_PARTITION_ID_SIZE : usize = 16;

pub struct PartitionId {
    name: Option<[u8; MAX_PARTITION_ID_SIZE]>,
}

impl PartitionId {
    pub fn default() -> Self {
        Self { name: None }
    }
}

pub struct Nvs {
    default_partition_initialized: bool,
}

#[non_exhaustive]
pub struct NvsPartition;

impl Nvs {
    pub fn init(_: NvsPeripherals) -> Self {
        Self {
            default_partition_initialized: false,
        }
    }

    pub fn init_partition(&mut self, id: PartitionId) -> Result<NvsPartition, NvsError> {
        let partition_init_result = match id.name {
            None => unsafe { nvs_flash_init() },
            _ => unimplemented!("Named nvs partitions not supported"),
        };

        match partition_init_result {
            esp_err_t_ESP_OK => Ok(NvsPartition),
            esp_err_t_ESP_ERR_NVS_NO_FREE_PAGES => Err(NvsError::PartitionCorrupted),
            esp_err_t_ESP_ERR_NVS_NOT_FOUND => Err(NvsError::PartitionNotFound),
            err => Err(NvsError::IdfError(err)),
        }
    }

    pub fn deinit_partition(&mut self, _: NvsPartition) {
        self.default_partition_initialized = false
    }

    pub fn erase_partition(&mut self, _id: PartitionId) -> Result<(), NvsError> {
        if self.default_partition_initialized {
            Err(NvsError::AlreadyInitialized)
        } else {
            match unsafe { nvs_flash_erase() } {
                esp_err_t_ESP_OK => Ok(()),
                esp_err_t_ESP_ERR_NVS_NOT_FOUND => Err(NvsError::PartitionNotFound),
                err => Err(IdfError(err)),
            }
        }
    }
}