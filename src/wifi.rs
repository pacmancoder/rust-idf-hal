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

/// Provides wifi control interface.
///
/// Can be obtained from
/// [WiFiHardware.initialize](struct.WiFiHardware.html#method.initialize).
pub struct WiFi {
    hardware: WiFiHardware,
    ap_configuration: Option<WiFiApConfiguration>,
    sta_configuration: Option<WiFiStaConfiguration>,
    started: bool,
}

/// Represents WiFi access point configuration.
///
/// Can be produced with
/// [WiFiApConfigurationBuilder](struct.WiFiApConfigurationBuilder.html)
pub struct WiFiApConfiguration {
    config: wifi_config_t,
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

/// Represents threshold for access point scanning
pub struct WiFiScanThreshold {
    data: wifi_scan_threshold_t,
}


/// WiFi scan threshold build error
pub enum WiFiScanThresholdBuildError {
    /// Invalid value of Rssi
    InvalidRssi,
}

/// Provides interface for WiFi scan threshold building
pub struct WiFiScanThresholdBuilder {
    rssi: Option<i8>,
    auth_mode: Option<WiFiAuthMode>,

    pending_error: Option<WiFiScanThresholdBuildError>
}

impl WiFiScanThresholdBuilder {
    pub fn new() -> Self {
        Self {
            rssi: None,
            auth_mode: None,
            pending_error: None
        }
    }

    /// Set minimal received
    /// [signal strength](https://en.wikipedia.org/wiki/Received_signal_strength_indication);
    /// Represented with negative value in db. If signal strength should be ignored on the scan,
    /// this value should not be set for the threshold
    pub fn min_signal_strength(mut self, rssi: i8) -> Self {
        if rssi >= 0 {
            self.pending_error = Some(WiFiScanThresholdBuildError::InvalidRssi);
        } else {
            self.rssi = Some(rssi);
        }

        self
    }

    /// Sets minimal auth mode strength for the WiFi (e.g. open network is weaker than wpa2)
    pub fn min_auth_mode(mut self, mode: WiFiAuthMode) -> Self {
        self.auth_mode = Some(mode);
        self
    }

    pub fn build(self) -> WiFiScanThreshold {
        WiFiScanThreshold {
            data : wifi_scan_threshold_t {
                rssi: self.rssi.unwrap_or(0),
                authmode: hal_to_sys::auth_mode(
                    self.auth_mode.unwrap_or(WiFiAuthMode::OpenNetwork)
                ),
            }
        }
    }
}

/// Provides interface for WiFi station mode configuration building.
///
/// After configuration has been built, [WiFiStaConfiguration](struct.WiFiStaConfiguration.html)
/// can be obtained from [build](#method.build) method
pub struct WiFiStaConfigurationBuilder {
    ssid: Option<[u8; 32]>,
    password: Option<[u8; 64]>,
    scan_method: wifi_scan_method_t,
    bssid: Option<[u8; 6]>,
    channel: u8,
    listen_interval: u16,
    sort_method: wifi_sort_method_t,
    scan_threshold: Option<WiFiScanThreshold>,

    pending_error: Option<WiFiStaConfigurationBuildError>,
}

mod hal_to_sys {
    use super::*;

    pub fn auth_mode(mode: WiFiAuthMode) -> u32 {
        match mode {
            WiFiAuthMode::OpenNetwork => wifi_auth_mode_t_WIFI_AUTH_OPEN,
            WiFiAuthMode::WpaPsk => wifi_auth_mode_t_WIFI_AUTH_WPA_PSK,
            WiFiAuthMode::Wpa2Psk => wifi_auth_mode_t_WIFI_AUTH_WPA2_PSK,
            WiFiAuthMode::WpaWpa2Psk => wifi_auth_mode_t_WIFI_AUTH_WPA_WPA2_PSK,
            WiFiAuthMode::Wpa2Enterprise => wifi_auth_mode_t_WIFI_AUTH_WPA2_ENTERPRISE,
            _ => unreachable!(),
        }
    }

    pub fn wifi_mode(mode: WiFiMode) -> u32 {
        match mode {
            WiFiMode::None => wifi_mode_t_WIFI_MODE_NULL,
            WiFiMode::Ap => wifi_mode_t_WIFI_MODE_AP,
            WiFiMode::Sta => wifi_mode_t_WIFI_MODE_STA,
            WiFiMode::Combined => wifi_mode_t_WIFI_MODE_APSTA,
        }
    }

    pub fn scan_method(method: WiFiScanMethod) -> u32 {
        match method {
            WiFiScanMethod::Fast => wifi_scan_method_t_WIFI_FAST_SCAN,
            WiFiScanMethod::AllChannel => wifi_scan_method_t_WIFI_ALL_CHANNEL_SCAN,
        }
    }

    pub fn sort_method(method: WiFiSortMethod) -> u32 {
        match method {
            WiFiSortMethod::BySignal => wifi_sort_method_t_WIFI_CONNECT_AP_BY_SIGNAL,
            WiFiSortMethod::BySecurity => wifi_sort_method_t_WIFI_CONNECT_AP_BY_SECURITY,
        }
    }
}

/// Represents WiFi station configuration.
///
/// Can be produced with
/// [WiFiStaConfigurationBuilder](struct.WiFiStaConfigurationBuilder.html)
pub struct WiFiStaConfiguration {
    config: wifi_config_t,
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
#[derive(Debug)]
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
    /// No configurations set
    ConfigurationNotSet,
    /// Internal IDF error
    IdfError(esp_err_t),
}

#[derive(Debug)]
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

/// WiFi STA configuration builder error
/// #[derive(Debug)]
pub enum WiFiStaConfigurationBuildError {
    /// Ssid is not sat and no bssid was provided
    SsidNotSet,
    /// Password not set, but threshold requested non-open network
    PasswordNotSet,
    /// Too long SSID - maximum allowed length is 32 **bytes** (not characters)
    TooLongSsid,
    /// Too long password - minimum allowed length is 64 **bytes** (not characters)
    TooLongPassword,
    /// Selected WiFiChannel is invalid (allowed channels are `1..=14`)
    InvalidWiFiChannel,
    /// Listen interval should be > 0
    InvalidListenInterval,
}

/// WiFi module operation mode
#[derive(Eq, PartialEq, Copy, Clone)]
pub enum WiFiMode {
    /// Disable both AP & STA
    None,
    /// Ap-only mode
    Ap,
    /// Sta-only mode
    Sta,
    /// Combined mode (AP + STA)
    Combined,
}

/// WiFi access points scan mode
pub enum WiFiScanMethod {
    /// Fast scan mode
    Fast,
    /// All-channel scan mode
    AllChannel,
}

/// WiFi access points sorting during scan process
pub enum WiFiSortMethod {
    /// Scan by higher signal
    BySignal,
    /// Scan by more secure auth mode
    BySecurity,
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
        let password_len = value.as_bytes().len();

        if password_len > 64 {
            self.pending_error = Some(WiFiApConfigurationBuildError::TooLongPassword);
        } else {
            let mut password : [u8; 64] = [0; 64];

            for (s, d) in value.as_bytes().iter().zip(password.iter_mut()) {
                *d = *s;
            }

            // Add null-terminator
            if password_len < 64 {
                password[password_len] = b'\0';
            }

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
            self.auth_mode = Some(hal_to_sys::auth_mode(value));
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

impl WiFiStaConfigurationBuilder {
    /// Creates new instance of WiFiStaConfigurationBuilder
    pub fn new() -> Self {
        Self {
            ssid: None,
            password: None,
            scan_method: wifi_scan_method_t_WIFI_FAST_SCAN,
            bssid: None,
            channel: 0,
            listen_interval: 0,
            sort_method: wifi_sort_method_t_WIFI_CONNECT_AP_BY_SIGNAL,
            scan_threshold: None,

            pending_error: None,
        }
    }


    /// Sets SSID for the STA
    ///
    /// Sets builder error if SSID is too long (>=32 **bytes**)
    pub fn ssid(mut self, value: &str) -> Self {
        let ssid_len = value.as_bytes().len();

        if value.as_bytes().len() > 32 {
            self.pending_error = Some(WiFiStaConfigurationBuildError::TooLongSsid);
        } else {
            let mut ssid : [u8; 32] = [0; 32];

            for (s, d) in value.as_bytes().iter().zip(ssid.iter_mut()) {
                *d = *s;
            }

            // Add terminating zero
            if value.as_bytes().len() < 32 {
                ssid[ssid_len] = b'\0';
            }

            self.ssid = Some(ssid);
        }

        self
    }

    /// Sets password for the AP authentication
    ///
    /// Sets builder error if password is too long (>=64 **bytes**)
    pub fn password(mut self, value: &str) -> Self {
        let password_len = value.as_bytes().len();

        if password_len > 64 {
            self.pending_error = Some(WiFiStaConfigurationBuildError::TooLongPassword);
        } else {
            let mut password : [u8; 64] = [0; 64];

            for (s, d) in value.as_bytes().iter().zip(password.iter_mut()) {
                *d = *s;
            }

            // Add null-terminator
            if password_len < 64 {
                password[password_len] = b'\0';
            }

            self.password = Some(password);
        }

        self
    }

    /// Changes access point scan mode for the station. Default is `WiFiScanMethod::Fast`
    pub fn scan_method(mut self, method: WiFiScanMethod) -> Self {
        self.scan_method = hal_to_sys::scan_method(method);

        self
    }

    /// Sets bssid of the point to increase connection speed of for connection to the access points
    /// with hidden ssid's
    pub fn bssid(mut self, value: [u8; 6]) -> Self {
        self.bssid = Some(value);
        self
    }

    /// Selects target access point channel. Do not set the channel directly to
    /// find it automatically
    ///
    /// Sets builder error, if provided channel is invalid (it should be in range 1..=14)
    pub fn channel(mut self, value: u8) -> Self {
        if value == 0 || value > 14 {
            self.pending_error = Some(WiFiStaConfigurationBuildError::InvalidWiFiChannel)
        } else {
            self.channel = value;
        }

        self
    }

    /// Changes access points scan interval. Measured in AP beacon intervals (e.g. if AP beacon
    /// interval was set to 100, and listen interval was set to 3 - resulting interval will be
    /// 300ms. Beacon interval could be changed by setting ap configuration. Default beacon
    /// interval is 100ms. Default listen interval is 3 (300ms)
    pub fn listen_interval(mut self, interval: u16) -> Self {
        if interval == 0 {
            self.pending_error = Some(WiFiStaConfigurationBuildError::InvalidListenInterval);
        } else {
            self.listen_interval = interval
        }

        self
    }

    /// Changes access points sort method for scanning. Default sort method is
    /// `WiFiSortMethod::BySignal`
    pub fn sort_method(mut self, method: WiFiSortMethod) -> Self {
        self.sort_method = hal_to_sys::sort_method(method);
        self
    }

    /// Changes access points scan threshold settings
    pub fn scan_threshold(mut self, threshold: WiFiScanThreshold) -> Self {
        self.scan_threshold = Some(threshold);
        self
    }

    /// Builds `WiFiStaCofiguration` or returns error.
    pub fn build(self) -> Result<WiFiStaConfiguration, WiFiStaConfigurationBuildError> {
        if let Some(err) = self.pending_error {
            return Err(err)
        }

        if self.ssid.is_none() && self.bssid.is_none() {
            return Err(WiFiStaConfigurationBuildError::SsidNotSet);
        }

        if let Some(ref threshold) = self.scan_threshold {
            if threshold.data.authmode != wifi_auth_mode_t_WIFI_AUTH_OPEN
                && self.password.is_none()
            {
                return Err(WiFiStaConfigurationBuildError::PasswordNotSet);
            }
        }

        Ok(WiFiStaConfiguration {
            config: wifi_config_t {
                sta: wifi_sta_config_t {
                    ssid: self.ssid.unwrap_or([0; 32]),
                    password: self.password.unwrap_or([0; 64]),
                    scan_method: self.scan_method,
                    bssid_set: self.bssid.is_some(),
                    bssid: self.bssid.unwrap_or([0; 6]),
                    channel: self.channel,
                    listen_interval: self.listen_interval,
                    sort_method: self.sort_method,
                    threshold: self.scan_threshold.map_or(wifi_scan_threshold_t {
                        rssi: 0,
                        authmode: wifi_auth_mode_t_WIFI_AUTH_OPEN,
                    }, |t| t.data),
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
    /// Returns [WiFi](struct.WiFi.html) on success
    ///
    /// Returns tuple containing error and self (to be able to try initialize again of gracefully
    /// return [WiFiPeripherals](../peripherals/struct.WiFiPeripherals.html) instance) when
    /// WiFi module initialization has been failed
    pub fn initialize(mut self) -> Result<WiFi, (WiFiInitializationError, Self)> {
        let initialization_result;
        unsafe {
            // TODO: deinit in wifi release
            tcpip_adapter_init();

            let wifi_init_config = WIFI_INIT_CONFIG_DEFAULT();
            initialization_result = esp_wifi_init(&wifi_init_config);
        }

        if initialization_result != esp_err_t_ESP_OK {
            Err((WiFiInitializationError::IdfError(initialization_result), self))
        } else {
            self.initialized = true;
            Ok(WiFi::new(self))
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
    #[allow(non_upper_case_globals)]
    match result {
        esp_err_t_ESP_OK => Ok(()),
        esp_err_t_ESP_ERR_WIFI_PASSWORD => Err(WiFiConfigurationError::InvalidWifiPassword),
        esp_err_t_ESP_ERR_WIFI_NVS => Err(WiFiConfigurationError::InternalNvsError),
        err => Err(WiFiConfigurationError::IdfError(err))
    }
}

impl WiFi {
    fn new(hardware: WiFiHardware)  -> Self {
        Self {
            hardware,
            ap_configuration: None,
            sta_configuration: None,
            started: false,
        }
    }

    /// Sets or changes WiFi access point configuration
    pub fn set_ap_config(&mut self, mut config: WiFiApConfiguration) -> &mut Self {
        self.ap_configuration = Some(config);
        self
    }

    /// Sets or changes WiFi station configuration
    pub fn set_sta_config(&mut self, mut config: WiFiStaConfiguration) -> &mut Self
    {
        self.sta_configuration = Some(config);
        self
    }

    /// Gracefully stops the WiFi and returns owned WiFiHardware
    pub fn downgrade(mut self) -> WiFiHardware {
        self.stop();
        self.hardware
    }

    /// Starts configured WiFi in STA/AP/STA+AP mode
    ///
    /// Selects required mode(STA/AP/STA+AP) based on the provided configurations,
    /// configures AP, STA and starts them.
    ///
    /// Returns error if WiFi configuration or startup have been failed.
    pub fn start(&mut self) -> Result<&mut Self, WiFiConfigurationError> {
        let mode = match (self.sta_configuration.is_some(), self.ap_configuration.is_some()) {
            (true, true) => WiFiMode::Combined,
            (true, false) => WiFiMode::Sta,
            (false, true) => WiFiMode::Sta,
            _ => {
                return Err(WiFiConfigurationError::ConfigurationNotSet);
            }
        };

        let set_mode_result = unsafe { esp_wifi_set_mode(hal_to_sys::wifi_mode(mode)) };

        if set_mode_result != esp_err_t_ESP_OK {
            return Err(WiFiConfigurationError::IdfError(set_mode_result))
        }

        if let Some(ref mut config) = self.sta_configuration {
            unsafe { set_wifi_config(esp_interface_t_ESP_IF_WIFI_STA, &mut config.config)?; }
        }

        if let Some(ref mut config) = self.ap_configuration {
            unsafe { set_wifi_config(esp_interface_t_ESP_IF_WIFI_AP, &mut config.config)?; }
        }

        let result = unsafe { esp_wifi_start() };

        #[allow(non_upper_case_globals)]
        match result {
            esp_err_t_ESP_OK => {
                self.started = true;
                Ok(self)
            },
            esp_err_t_ESP_ERR_NO_MEM => Err(WiFiConfigurationError::NoMemory),
            esp_err_t_ESP_ERR_INVALID_ARG => Err(WiFiConfigurationError::InvalidArgument),
            esp_err_t_ESP_ERR_WIFI_CONN => Err(WiFiConfigurationError::ConnectionEstablishmentFailed),
            err => Err(WiFiConfigurationError::IdfError(err))
        }
    }

    /// Gracefully Stops wifi AP/STA
    pub fn stop(&mut self) -> &mut Self {
        if self.started {
            unsafe { esp_wifi_stop(); }
            self.started = false;
        }

        self
    }

    /// Performs connection to the STA
    pub fn connect(&mut self) -> Result<&mut Self, WiFiConfigurationError> {
        let err = unsafe { esp_wifi_connect() };
        if err != esp_err_t_ESP_OK {
            Err(WiFiConfigurationError::IdfError(err))
        } else {
            Ok(self)
        }
    }

    pub fn switch_sta_to_bgn_mode(&mut self) -> Result<&mut Self, WiFiConfigurationError> {
        let err = unsafe { esp_wifi_set_protocol(
            esp_interface_t_ESP_IF_WIFI_STA,
            (WIFI_PROTOCOL_11B | WIFI_PROTOCOL_11G | WIFI_PROTOCOL_11N) as u8
        ) };

        match err {
            esp_err_t_ESP_OK => Ok(self),
            err => Err(WiFiConfigurationError::IdfError(err)),
        }
    }
}