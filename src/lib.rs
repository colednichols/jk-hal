// #![allow(unused)]
#![no_std]
use core::{fmt, marker::PhantomData};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ecu<T: LocalIdentifier> {
    pub tx_id: u32,
    pub rx_id: u32,
    _marker: PhantomData<T>,
}

impl Ecu<InstrumentCluster> {
    pub const INSTRUMENT_CLUSTER: Ecu<InstrumentCluster> = Ecu {
        tx_id: 0x6A0,
        rx_id: 0x514,
        _marker: PhantomData,
    };
}

impl Ecu<FrontControl> {
    pub const FRONT_CONTROL: Ecu<FrontControl> = Ecu {
        tx_id: 0x620,
        rx_id: 0x504,
        _marker: PhantomData,
    };
}

impl<T: LocalIdentifier> Ecu<T> {
    pub fn build_request(&self, cmd: &UdsCommand<T>) -> (u32, [u8; 8]) {
        let (payload, len) = cmd.to_payload();
        
        let mut frame = [0x00; 8];
        
        let safe_len = len.min(7); 
        frame[0] = safe_len as u8; 
        
        frame[1..1 + safe_len].copy_from_slice(&payload[..safe_len]);
        
        (self.tx_id, frame)
    }
}

pub trait LocalIdentifier {
    fn id(&self) -> u8;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstrumentCluster {
    Illum,
    LampDome,
    IndicatorABS,
    IndicatorAC,
    BTSI,
}

impl LocalIdentifier for InstrumentCluster {
    fn id(&self) -> u8 {
        match self {
            Self::Illum => 0x10,
            Self::LampDome => 0x11,
            Self::IndicatorABS => 0x30,
            Self::IndicatorAC => 0x55,
            Self::BTSI => 0x1E,
        }
    }
}

// Reverse matching for rx
impl TryFrom<u8> for InstrumentCluster {
    type Error = UdsError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x10 => Ok(Self::Illum),
            0x11 => Ok(Self::LampDome),
            0x30 => Ok(Self::IndicatorABS),
            0x55 => Ok(Self::IndicatorAC),
            0x1E => Ok(Self::BTSI),
            _ => Err(UdsError::UnknownServiceId(val)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrontControl {
    LFog,
    RFog,
    LrTurn,
    RrTurn,
    Park,
    Reverse,
    LLow,
    RLow,
    LHigh,
    RHigh,
    LrStop,
    RrStop,
    Horn,
    LfTurn,
    RfTurn,
}

impl LocalIdentifier for FrontControl {
    fn id(&self) -> u8 {
        match self {
            Self::LFog => 0xA0,
            Self::RFog => 0xA1,
            Self::LrTurn => 0xA2,
            Self::RrTurn => 0xA3,
            Self::Park => 0xA4,
            Self::Reverse => 0xA5,
            Self::LLow => 0xA8,
            Self::RLow => 0xA9,
            Self::LHigh => 0xAA,
            Self::RHigh => 0xAB,
            Self::LrStop => 0xAE,
            Self::RrStop => 0xAF,
            Self::Horn => 0xAD,
            Self::LfTurn => 0x14,
            Self::RfTurn => 0x15,
        }
    }
}

// Reverse matching for rx
impl TryFrom<u8> for FrontControl {
    type Error = UdsError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0xA0 => Ok(Self::LFog),
            0xA1 => Ok(Self::RFog),
            0xA2 => Ok(Self::LrTurn),
            0xA3 => Ok(Self::RrTurn),
            0xA4 => Ok(Self::Park),
            0xA5 => Ok(Self::Reverse),
            0xA8 => Ok(Self::LLow),
            0xA9 => Ok(Self::RLow),
            0xAA => Ok(Self::LHigh),
            0xAB => Ok(Self::RHigh),
            0xAE => Ok(Self::LrStop),
            0xAF => Ok(Self::RrStop),
            0xAD => Ok(Self::Horn),
            0x14 => Ok(Self::LfTurn),
            0x15 => Ok(Self::RfTurn),
            _ => Err(UdsError::UnknownServiceId(value)),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionControlParam {
    End = 0x00,
    Enhanced = 0x92,
}

impl SessionControlParam {
    pub fn to_byte(&self) -> u8 {
        *self as u8
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IOControlParam {
    End,
    Reset,
    Freeze,
    Set(u8), // carry value to set
    Read,
}

impl IOControlParam {
    // Return the bytes and how many bytes are valid
    pub fn to_bytes(&self) -> ([u8; 2], usize) {
        match self {
            Self::End => ([0x00, 0x00], 1),
            Self::Reset => ([0x01, 0x00], 1),
            Self::Freeze => ([0x02, 0x00], 1),
            Self::Set(val) => ([0x07, *val], 2), // 2 bytes used
            Self::Read => ([0x07, 0x00], 1),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TesterPresentParam {
    End = 0x00,
    KeepAlive = 0x01,
}
impl TesterPresentParam {
    pub fn to_byte(&self) -> u8 {
        *self as u8
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UdsServiceId {
    DiagnosticSessionControl = 0x10,
    InputOutputControl = 0x30, // Using 0x30 as implemented for the JK
    TesterPresent = 0x3E,
}

impl TryFrom<u8> for UdsServiceId {
    type Error = UdsError;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            0x10 => Ok(Self::DiagnosticSessionControl),
            0x30 => Ok(Self::InputOutputControl),
            0x3E => Ok(Self::TesterPresent),
            _ => Err(UdsError::UnknownServiceId(val)),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum UdsCommand<T: LocalIdentifier> {
    SessionControl(SessionControlParam),
    InputOutputControl(T, IOControlParam), // T is strongly-typed actuator
    TesterPresent(TesterPresentParam),
}

impl<T: LocalIdentifier> UdsCommand<T> {
pub fn to_payload(&self) -> ([u8; 7], usize) {
        let mut payload = [0x00; 7];

        match self {
            UdsCommand::SessionControl(param) => {
                payload[0] = UdsServiceId::DiagnosticSessionControl as u8;
                payload[1] = param.to_byte(); // Note: Your ledger correction of 0x92 is already handled by the enum!
                (payload, 2)
            }
            UdsCommand::InputOutputControl(actuator, param) => {
                payload[0] = UdsServiceId::InputOutputControl as u8;
                payload[1] = actuator.id();
                
                let (p_bytes, p_len) = param.to_bytes();
                // Idiomatic, panic-free copying
                payload[2..2 + p_len].copy_from_slice(&p_bytes[..p_len]);
                
                (payload, 2 + p_len)
            }
            UdsCommand::TesterPresent(param) => {
                payload[0] = UdsServiceId::TesterPresent as u8;
                payload[1] = param.to_byte();
                (payload, 2)
            }
        }
    }

    pub fn io_set(actuator: T, val: u8) -> Self {
        Self::InputOutputControl(actuator, IOControlParam::Set(val))
    }

    pub fn io_off(actuator: T) -> Self {
        Self::InputOutputControl(actuator, IOControlParam::Set(0x00))
    }

    pub fn io_end(actuator: T) -> Self {
        Self::InputOutputControl(actuator, IOControlParam::End)
    }

    pub fn keep_alive() -> Self {
        Self::TesterPresent(TesterPresentParam::KeepAlive)
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Nrc {
    ServiceNotSupported = 0x11,
    ConditionsNotCorrect = 0x22,
    RequestOutOfRange = 0x31,
    SecurityAccessDenied = 0x33,
    ResponsePending = 0x78,
    Unknown(u8),
}

impl From<u8> for Nrc {
    fn from(val: u8) -> Self {
        match val {
            0x11 => Self::ServiceNotSupported,
            0x22 => Self::ConditionsNotCorrect,
            0x31 => Self::RequestOutOfRange,
            0x33 => Self::SecurityAccessDenied,
            0x78 => Self::ResponsePending,
            _ => Self::Unknown(val),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UdsError {
    WrongTargetId,
    InvalidPciByte,
    InvalidLength,
    UnknownServiceId(u8),
}

impl fmt::Display for UdsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        match self {
            Self::WrongTargetId => write!(f, "CAN ID does not match ECU RX ID"),
            Self::InvalidPciByte => write!(f, "Not an ISO-TP Single Frame"),
            Self::InvalidLength => write!(f, "ISO-TP length out of bounds"),
            Self::UnknownServiceId(id) => write!(f, "Unknown UDS Service ID: {:#04X}", id),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UdsResponse {
    Positive(UdsServiceId, [u8; 6], usize), // Service, Payload, Length
    Negative(UdsServiceId, Nrc),            // Service, NRC Code
}

impl core::error::Error for UdsError {}

pub struct SingleFrameParser;

impl SingleFrameParser {
    pub fn parse(target_ecu_rx: u32, frame_id: u32, data: &[u8; 8]) -> Result<UdsResponse, UdsError> {
        // 1. Validate routing
        if frame_id != target_ecu_rx {
            return Err(UdsError::WrongTargetId);
        }

        // 2. Validate ISO-TP Single Frame PCI
        if data[0] & 0xF0 != 0x00 {
            return Err(UdsError::InvalidPciByte);
        }

        // 3. Validate Length limits
        let len = (data[0] & 0x0F) as usize;
        if len == 0 || len > 7 {
            return Err(UdsError::InvalidLength);
        }

        let service_id = data[1];
        
        // 4. Handle Negative Response Code (NRC)
        if service_id == 0x7F {
            if len < 3 { // An NRC frame must have at least 3 payload bytes (7F, Failed ID, NRC)
                return Err(UdsError::InvalidLength);
            }
            
            // Map the raw byte back to our enum safely using TryFrom
            let failed_service = UdsServiceId::try_from(data[2])?;
            let nrc = Nrc::from(data[3]);
            
            return Ok(UdsResponse::Negative(failed_service, nrc));
        }

        // 5. Handle Positive Response
        // Positive responses are the original Service ID + 0x40. 
        // checked_sub prevents underflow panics if hardware sends a malformed byte.
        let original_service_byte = service_id.checked_sub(0x40)
            .ok_or(UdsError::UnknownServiceId(service_id))?;
            
        let original_service = UdsServiceId::try_from(original_service_byte)?;
        
        let mut payload = [0x00; 6];
        let payload_len = len - 1; // Subtract the 1 byte used by the Service ID
        
        if payload_len > 0 {
            // Idiomatic slice copying based on dynamic length
            payload[..payload_len].copy_from_slice(&data[2..1 + len]);
        }

        Ok(UdsResponse::Positive(original_service, payload, payload_len))
    }
}