#[derive(Debug, Clone, Copy)]
pub enum ResponseCodes {
    Success = 0,
    InvalidRequest,
    NoSessionExists,
}

impl ResponseCodes {
    pub fn from_bytes(buff: &mut [u8]) -> Self {
        match buff[0] {
            0 => ResponseCodes::Success,
            1 => ResponseCodes::InvalidRequest,
            2 => ResponseCodes::NoSessionExists,
            _ => panic!("Invalid Response Code")
        }
    }
}

impl Transmittable for ResponseCodes {
    fn to_bytes(&self) -> Vec<u8> {
        return vec![*self as u8];
    }
}

#[derive(Debug)]
pub struct SessionStatusMessage {
    work_seconds: u32,
    rest_seconds: u32,
    time_remaining: u32,
    state: SessionState,
}

pub trait Transmittable {
    fn to_bytes(&self) -> Vec<u8>;
}

impl SessionStatusMessage {
    pub fn new(
        work_seconds: u32,
        rest_seconds: u32,
        time_remaining: u32,
        state: SessionState,
    ) -> Self {
        Self {
            work_seconds,
            rest_seconds,
            time_remaining,
            state,
        }
    }

    /// Read in bytes from tx and create Message
    pub fn from_bytes(buff: &mut [u8]) -> Self {
        let (work_bytes, rest) = buff.split_at(std::mem::size_of::<u32>());
        let work_seconds = u32::from_be_bytes(work_bytes.try_into().unwrap());

        let (rest_bytes, rest) = rest.split_at(std::mem::size_of::<u32>());
        let rest_seconds = u32::from_be_bytes(rest_bytes.try_into().unwrap());

        let (time_bytes, rest) = rest.split_at(std::mem::size_of::<u32>());
        let time_remaining = u32::from_be_bytes(time_bytes.try_into().unwrap());

        let state = match rest[0] {
            0 => SessionState::Working,
            1 => SessionState::Resting,
            _ => panic!("INVALID STATE!"),
        };

        Self {
            work_seconds,
            rest_seconds,
            time_remaining,
            state,
        }
    }
}

impl Transmittable for SessionStatusMessage {
    /// Convert the status into Bytes for tx
    fn to_bytes(&self) -> Vec<u8> {
        let mut buff: Vec<u8> = Vec::with_capacity(17);
        buff.extend_from_slice(&self.work_seconds.to_be_bytes());
        buff.extend_from_slice(&self.rest_seconds.to_be_bytes());
        buff.extend_from_slice(&self.time_remaining.to_be_bytes());
        buff.push(self.state as u8);
        buff
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum SessionState {
    Working,
    Resting,
}

#[cfg(test)]
mod tests {}
