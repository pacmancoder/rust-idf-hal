//! This module provides access to wifi-related types.
//! To use WiFiHarware type, defined in this module, WiFiPeripherals
//! instance should be obtained first.
//!
//! # Examples
//! ```no_run
//! # use idf_hal::{
//! #     peripherals::Peripherals,
//! #     wifi::{WiFiAuthMode, WiFiHardware, WiFiApConfigurationBuilder},
//! # };
//!
//! let peripherals = Peripherals::take().unwrap();
//!
//! let ap_config = WiFiApConfigurationBuilder::new()
//!     .ssid("Hello, world!")
//!     .auth_mode(WiFiAuthMode::WpaWpa2Psk)
//!     .password("mypassword")
//!     .build().ok().unwrap();
//!
//! let wifi_configurator = WiFiHardware::new(peripherals.wifi)
//!     .initialize()
//!     .ok().unwrap();
//!
//! let wifi = wifi_configurator
//!     .set_ap_config(ap_config)
//!     .start()
//!     .ok().unwrap();
//! ```
use crate::peripherals::WiFiPeripherals;

use idf_sys:: {
    wifi::*,
    error::*,
    network_adapter::*,
};

/// Represents WiFi hardware instance.
///
/// Should be be constructed from
/// [WiFiPeripherals](../peripherals/struct.WiFiPeripherals.html)
///
/// Provides hardware initialization/deinitialization methods.
///
/// Transforms into
/// [WiFiConfigurator](struct.WiFiConfigurator.html) after [initialization](#method.initialize).
/// for further WiFi setup.
///
/// Using [deinitialize](#method.deinitialize) it can be downgraded back to `WiFiPeripherals`
pub struct WiFiHardware {
    initialized: bool,
    peripherals: WiFiPeripherals,
}

/// Provides facilities to configure WiFi in AP/STA/both mode.
///
/// Can be obtained from
/// [WiFiHardware.initialize](struct.WiFiHardware.html#method.initialize).
pub struct WiFiConfigurator {
    wifi: WiFi,
}

/// Represents WiFi access point configuration.
///
/// Can be produced with
/// [WiFiApConfigurationBuilder](struct.WiFiApConfigurationBuilder.html)
pub struct WiFiApConfiguration {
    config: wifi_config_t
}


/// Provides interface for WiFi access point configuration building.
///
/// After configuration has been built, [WiFiApConfiguration](struct.WiFiApConfiguration.html)
/// can be obtained from [build](#method.build) method
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

/// Represents WiFi station configuration.
///
/// Can be produced with
/// [WiFiStaConfigurationBuilder](struct.WiFiStaConfigurationBuilder.html)
pub struct WiFiStaConfiguration {
    config: wifi_config_t
}

/// Represents configured WiFi adapter. When reconfiguration is needed, it should be
/// [stopped](#method.stop) in order to downgrade back to WiFiHardware
pub struct WiFi {
    hardware: WiFiHardware,
    ap_configuration: Option<WiFiApConfiguration>,
    sta_configuration: Option<WiFiStaConfiguration>,
}

/// WiFi authentication mode
pub enum WiFiAuthMode {
    /// Open network without protection
    OpenNetwork,
    /// WEP authentication
    Wep,
    /// WPA PSK authentication
    WpaPsk,
    /// WPA2 PSK authentication
    Wpa2Psk,
    /// WPA PSK or WPA2 PSK authentication
    WpaWpa2Psk,
    /// WPA2 Enterprise authentication
    Wpa2Enterprise,
}

/// WiFi initialization error
pub enum WiFiInitializationError {
    /// Internal IDF Error
    IdfError(esp_err_t),
}

/// WiFi configuration error
///
/// Produced when trying to start WiFi adapter
pub enum WiFiConfigurationError {
    /// Provided WiFi password is invalid
    InvalidWifiPassword,
    /// Internal nvs module error
    InternalNvsError,
    /// Invalid argument in configuration
    InvalidArgument,
    /// No available memory to perform wifi structures allocation
    NoMemory,
    /// Connection establishment failed
    ConnectionEstablishmentFailed,
    /// Internal IDF error
    IdfError(esp_err_t),
}

pub enum WiFiApConfigurationBuildError {
    /// SSID is not set, although network set ad non-hidden
    SsidNotSet,
    /// Password set, although network has non-open auth type
    PasswordNotSet,
    /// Auth mode was not selected
    AuthModeNotSet,
    /// Too long SSID - maximum allowed length is 32 **bytes** (not characters)
    TooLongSsid,
    /// Too long password - minimum allowed length is 64 **bytes** (not characters)
    TooLongPassword,
    /// Selected WiFiChannel is invalid (allowed channels are `1..=14`)
    InvalidWiFiChannel,
    /// Invalid beacon interval - allowed interval is 100..=60000 ms
    InvalidBeaconInterval,
    /// Invalid max connections - allowed count is 1..=4
    InvalidMaxConnections,
    /// Not supported auth mode (e.g. WEP is not available for AP)
    AuthModeNotSupported,
}

impl WiFiApConfigurationBuilder {
    /// Creates new `WiFiApConfigurationBuilder` instance
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

    /// Sets SSID for the AP
    ///
    /// Sets builder error if SSID is too long (>=32 **bytes**)
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

    /// Sets password for the AP authentication
    ///
    /// Sets builder error if password is too long (>=64 **bytes**)
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

    /// Selected access point auth mode.
    ///
    /// Sets builder error when WEP mode selected - it is not supported for the AP mode
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

    /// Selects access point channel.
    ///
    /// Sets builder error, if provided channel is invalid (it should be in range 1..=14)
    pub fn channel(mut self, value: u8) -> Self {
        if value == 0 || value > 14 {
            self.pending_error = Some(WiFiApConfigurationBuildError::InvalidWiFiChannel)
        } else {
            self.channel = value;
        }

        self
    }

    /// Makes access point hidden
    pub fn hidden(mut self, value: bool) -> Self {
        self.ssid_hidden = if value { 1 } else { 0 };
        self
    }

    /// Sets max connections count
    ///
    /// Value should be between 1..=4, otherwise builder will be poisoned with error
    pub fn max_connections(mut self, value: u8) -> Self {
        if value == 0 || value > 4 {
            self.pending_error = Some(WiFiApConfigurationBuildError::InvalidMaxConnections)
        } else {
            self.max_connections = value;
        }

        self
    }

    /// Sets access point beacon interval (in ms)
    ///
    /// Sets builder error if value is not in range (100..=60000)
    pub fn beacon_interval(mut self, value: u16) -> Self {
        if value < 100 || value > 60000 {
            self.pending_error = Some(WiFiApConfigurationBuildError::InvalidBeaconInterval);
        } else {
            self.beacon_interval = value;
        }

        self
    }

    /// Builds WiFi access point configuration
    ///
    /// Returns error if any of the fields have been set incorrectly
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

    /// Creates new WiFi hardware instance from
    /// [WiFiPeripherals](../peripherals/struct.WiFiPeripherals.html)
    pub fn new(peripherals: WiFiPeripherals) -> WiFiHardware {
        Self { initialized: false, peripherals }
    }

    /// Performs WiFi adapter initialization with the default options.
    ///
    /// Returns [WiFiConfigurator](struct.WiFiConfigurator.html) on success
    ///
    /// Returns tuple containing error and self (to be able to try initialize again of gracefully
    /// return [WiFiPeripherals](../peripherals/struct.WiFiPeripherals.html) instance) when
    /// WiFi module initialization has been failed
    pub fn initialize(mut self) -> Result<WiFiConfigurator, (WiFiInitializationError, Self)> {
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

    /// Deinitializes WiFi module
    ///
    /// Returns previously owned WiFiPeripherals
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

    /// Sets WiFi access point configuration
    pub fn set_ap_config(mut self, config: WiFiApConfiguration) -> Self {
        self.wifi.ap_configuration = Some(config);
        self
    }

    /// Sets WiFi stantion configuration
    pub fn set_sta_config(mut self, config: WiFiStaConfiguration) -> Self {
        self.wifi.sta_configuration = Some(config);
        self
    }

    /// Releases previously owned WiFiHardware
    ///
    /// Can be used to gracefully shutdown wifi adapter
    pub fn release_hardware(self) -> WiFiHardware {
        self.wifi.hardware
    }

    /// Starts configured WiFi in STA/AP/STA+AP mode
    ///
    /// Selects required mode(STA/AP/STA+AP) based on the provided configurations,
    /// configures AP, STA and starts them.
    ///
    /// Returns error if WiFi configuration or startup have been failed.
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
    /// Stops WiFi (both AP and STA) and returns the WiFiHardware to gracefully shutdown wifi
    /// module.
    pub fn stop(self) -> WiFiHardware {
        // ESP_ERR_WIFI_NOT_INIT is not possible here, esp_wifi_stop should always return ESP_OK
        unsafe {
            let result = esp_wifi_stop();
            assert_eq!(esp_err_t_ESP_OK, result);
        }

        self.hardware
    }
}
