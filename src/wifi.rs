use crate::peripherals::WiFiPeripherals;

use idf_sys:: {
    wifi::*,
    error::*,
    network_adapter::*,
};

pub struct WiFiHardware {
    initialized: bool,
    peripherals: WiFiPeripherals,
}

pub struct WiFiConfigurator {
    wifi: WiFi,
}

pub struct WiFiApConfiguration {
    config: wifi_config_t
}

pub struct WiFiApConfigurationBuilder {
    ssid: Option<[u8; 32usize]>,
    password: Option<[u8; 64usize]>,
    ssid_len: u8,
    channel: u8,
    auth_mode: Option<wifi_auth_mode_t>,
    ssid_hidden: u8,
    max_connections: u8,
    beacon_interval: u16,

    pending_error: Option<WiFiApConfigurationBuildError>,
}

pub struct WiFiStaConfiguration {
    config: wifi_config_t
}

pub struct WiFi {
    hardware: WiFiHardware,
    ap_configuration: Option<WiFiApConfiguration>,
    sta_configuration: Option<WiFiStaConfiguration>,
}

pub enum WiFiAuthMode {
    OpenNetwork,
    Wep,
    WpaPsk,
    Wpa2Psk,
    WpaWpa2Psk,
    Wpa2Enterprise,
}

pub enum WiFiInitializationError {
    AlreadyInitialized,
    IdfError(esp_err_t),
}

pub enum WiFiConfigurationError {
    InvalidWifiPassword,
    InternalNvsError,
    InvalidArgument,
    NoMemory,
    ConnectionEstablishmentFailed,
    IdfError(esp_err_t),
}

pub enum WiFiApConfigurationBuildError {
    SsidNotSet,
    PasswordNotSet,
    AuthModeNotSet,
    TooLongSsid,
    TooLongPassword,
    InvalidWiFiChannel,
    InvalidBeaconInterval,
    InvalidMaxConnections,
    AuthModeNotSupported,
}

impl WiFiApConfigurationBuilder {
    pub fn new() -> Self {
        Self {
            ssid: None,
            password: None,
            ssid_len: 0,
            channel: 0,
            auth_mode: None,
            ssid_hidden: 0,
            max_connections: 4,
            beacon_interval: 100,

            pending_error: None
        }
    }

    pub fn ssid(mut self, value: &str) -> Self {
        if value.as_bytes().len() > 32 {
            self.pending_error = Some(WiFiApConfigurationBuildError::TooLongSsid);
        } else {
            let mut ssid : [u8; 32] = [0; 32];

            for (s, d) in value.as_bytes().iter().zip(ssid.iter_mut()) {
                *d = *s;
            }

            self.ssid = Some(ssid);
            self.ssid_len = value.as_bytes().len() as u8;
        }

        self
    }

    pub fn password(mut self, value: &str) -> Self {
        // Password should contain null terminator
        if value.as_bytes().len() + 1 > 64 {
            self.pending_error = Some(WiFiApConfigurationBuildError::TooLongPassword);
        } else {
            let mut password : [u8; 64] = [0; 64];

            for (s, d) in value.as_bytes().iter().zip(password.iter_mut()) {
                *d = *s;
            }

            password[value.as_bytes().len()] = b'\0';

            self.password = Some(password);
        }

        self
    }

    pub fn auth_mode(mut self, value: WiFiAuthMode) -> Self {
        if let WiFiAuthMode::Wep = value {
            self.pending_error = Some(WiFiApConfigurationBuildError::AuthModeNotSupported);
        } else {
            self.auth_mode = Some(match value {
                WiFiAuthMode::OpenNetwork => wifi_auth_mode_t_WIFI_AUTH_OPEN,
                WiFiAuthMode::WpaPsk => wifi_auth_mode_t_WIFI_AUTH_WPA_PSK,
                WiFiAuthMode::Wpa2Psk => wifi_auth_mode_t_WIFI_AUTH_WPA2_PSK,
                WiFiAuthMode::WpaWpa2Psk => wifi_auth_mode_t_WIFI_AUTH_WPA_WPA2_PSK,
                WiFiAuthMode::Wpa2Enterprise => wifi_auth_mode_t_WIFI_AUTH_WPA2_ENTERPRISE,
                _ => unreachable!(),
            });
        }

        self
    }

    pub fn channel(mut self, value: u8) -> Self {
        if value == 0 || value > 14 {
            self.pending_error = Some(WiFiApConfigurationBuildError::InvalidWiFiChannel)
        } else {
            self.channel = value;
        }

        self
    }

    pub fn hidden(mut self, value: bool) -> Self {
        self.ssid_hidden = if value { 1 } else { 0 };
        self
    }

    pub fn max_connections(mut self, value: u8) -> Self {
        if value == 0 || value > 4 {
            self.pending_error = Some(WiFiApConfigurationBuildError::InvalidMaxConnections)
        } else {
            self.max_connections = value;
        }

        self
    }

    pub fn beacon_interval(mut self, value: u16) -> Self {
        if value < 100 || value > 60000 {
            self.pending_error = Some(WiFiApConfigurationBuildError::InvalidBeaconInterval);
        } else {
            self.beacon_interval = value;
        }

        self
    }

    pub fn build(self) -> Result<WiFiApConfiguration, WiFiApConfigurationBuildError> {
        if let Some(err) = self.pending_error {
            return Err(err);
        }

        if self.ssid.is_none() && self.ssid_hidden == 0 {
            return Err(WiFiApConfigurationBuildError::SsidNotSet);
        }

        if self.auth_mode.is_none() {
            return Err(WiFiApConfigurationBuildError::AuthModeNotSet);
        }

        if self.password.is_none() && self.auth_mode.unwrap() != wifi_auth_mode_t_WIFI_AUTH_OPEN {
            return Err(WiFiApConfigurationBuildError::PasswordNotSet);
        }

        Ok(WiFiApConfiguration {
            config: wifi_config_t {
                ap: wifi_ap_config_t {
                    ssid: self.ssid.unwrap_or([0; 32]),
                    password: self.password.unwrap_or([0; 64]),
                    ssid_len: self.ssid_len,
                    channel: self.channel,
                    authmode: self.auth_mode.unwrap(),
                    ssid_hidden: self.ssid_hidden,
                    max_connection: self.max_connections,
                    beacon_interval: self.beacon_interval,
                }
            }
        })
    }
}

impl WiFiHardware {
    pub fn new(peripherals: WiFiPeripherals) -> WiFiHardware {
        Self { initialized: false, peripherals }
    }

    pub fn initialize(mut self) -> Result<WiFiConfigurator, (WiFiInitializationError, Self)> {
        if self.initialized {
            return Err((WiFiInitializationError::AlreadyInitialized, self))
        }

        let mut initialization_result = esp_err_t_ESP_OK;
        unsafe {
            let wifi_init_config = WIFI_INIT_CONFIG_DEFAULT();
            initialization_result = esp_wifi_init(&wifi_init_config);
        }

        if initialization_result != esp_err_t_ESP_OK {
            Err((WiFiInitializationError::IdfError(initialization_result), self))
        } else {
            self.initialized = true;
            Ok(WiFiConfigurator::new(self))
        }
    }

    pub fn deinitialize(self) -> WiFiPeripherals
    {
        if self.initialized {
            // according to idf sources, esp_wifi_deinit should always return ESP_OK
            let result = unsafe { esp_wifi_deinit() };
            assert_eq!(esp_err_t_ESP_OK, result);
        }

        self.peripherals
    }
}

unsafe fn set_wifi_config(interface: esp_interface_t, config: &mut wifi_config_t
) -> Result<(), WiFiConfigurationError> {
    let result = esp_wifi_set_config(interface, config);
    // ESP_ERR_WIFI_NOT_INIT, ESP_ERR_INVALID_ARG and ESP_ERR_WIFI_IF are not possible
    // here due to idf-hal design
    match result {
        esp_err_t_ESP_OK => Ok(()),
        esp_err_t_ESP_ERR_WIFI_PASSWORD => Err(WiFiConfigurationError::InvalidWifiPassword),
        esp_err_t_ESP_ERR_WIFI_NVS => Err(WiFiConfigurationError::InternalNvsError),
        err => Err(WiFiConfigurationError::IdfError(err))
    }
}

fn set_wifi_mode(ap: bool, sta: bool) -> Result<(), WiFiConfigurationError> {
    let mode = match ( ap, sta) {
        (false, false) => wifi_mode_t_WIFI_MODE_NULL,
        (true, false) => wifi_mode_t_WIFI_MODE_AP,
        (false, true) => wifi_mode_t_WIFI_MODE_STA,
        (true, true) => wifi_mode_t_WIFI_MODE_APSTA,
    };

    let result = unsafe { esp_wifi_set_mode(mode) };
    // ESP_ERR_WIFI_NOT_INIT and ESP_ERR_INVALID_ARG are not possible here due to
    // idf-hal design, only possible error is internal idf error
    if result != esp_err_t_ESP_OK {
        Err(WiFiConfigurationError::IdfError(result))
    } else {
        Ok(())
    }
}

fn start_wifi() -> Result<(), WiFiConfigurationError> {
    let result = unsafe { esp_wifi_start() };
    match result {
        esp_err_t_ESP_OK => Ok(()),
        esp_err_t_ESP_ERR_NO_MEM => Err(WiFiConfigurationError::NoMemory),
        esp_err_t_ESP_ERR_INVALID_ARG => Err(WiFiConfigurationError::InvalidArgument),
        esp_err_t_ESP_ERR_WIFI_CONN => Err(WiFiConfigurationError::ConnectionEstablishmentFailed),
        err => Err(WiFiConfigurationError::IdfError(err))
    }
}

impl WiFiConfigurator {

    fn new(hardware: WiFiHardware)  -> WiFiConfigurator {
        Self {
            wifi: WiFi { hardware, ap_configuration: None, sta_configuration: None }
        }
    }

    pub fn set_ap_config(mut self, config: WiFiApConfiguration) -> Self {
        self.wifi.ap_configuration = Some(config);
        self
    }

    pub fn set_sta_config(mut self, config: WiFiStaConfiguration) -> Self {
        self.wifi.sta_configuration = Some(config);
        self
    }

    pub fn release_hardware(self) -> WiFiHardware {
        self.wifi.hardware
    }

    pub fn start(mut self) -> Result<WiFi, WiFiConfigurationError> {
        self.set_required_mode()?;
        self.set_required_configurations()?;
        start_wifi()?;

        Ok(self.wifi)
    }

    fn set_required_mode(&self) -> Result<(), WiFiConfigurationError> {
        set_wifi_mode(
            self.wifi.ap_configuration.is_some(),
            self.wifi.sta_configuration.is_some()
        )
    }

    fn set_required_configurations(&mut self) -> Result<(), WiFiConfigurationError> {
        unsafe {
            if let Some(ref mut ap_configuration) = self.wifi.ap_configuration {
                set_wifi_config(esp_interface_t_ESP_IF_WIFI_AP, &mut ap_configuration.config)?;
            }

            if let Some(ref mut sta_configuration) = self.wifi.sta_configuration {
                set_wifi_config(esp_interface_t_ESP_IF_WIFI_STA, &mut sta_configuration.config)?;
            }
        }

        Ok(())
    }
}

impl WiFi {
    pub fn stop(self) -> WiFiHardware {
        // ESP_ERR_WIFI_NOT_INIT is not possible here, esp_wifi_stop should always return ESP_OK
        unsafe {
            let result = esp_wifi_stop();
            assert_eq!(esp_err_t_ESP_OK, result);
        }

        self.hardware
    }
}