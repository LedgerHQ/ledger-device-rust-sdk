use heapless::{String, Vec};

#[derive(Debug, Copy, Clone)]
pub struct FieldElement {
    pub value: [u8; 32]
}

impl FieldElement {

    pub const INVOKE: FieldElement = FieldElement {
        value: [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x69, 0x6e, 0x76, 0x6f, 0x6b, 0x65
        ]
    };

    pub const ZERO: FieldElement = FieldElement {
        value: [0u8; 32]
    };

    pub const ONE: FieldElement = FieldElement {
        value: [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01
        ]
    };

    pub const TWO: FieldElement = FieldElement {
        value: [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02
        ]
    };

    pub fn new() -> Self {
        Self {
            value: [0u8; 32]
        }
    }

    pub fn clear(&mut self) {
        self.value.fill(0);
    }
}

impl From<&[u8]> for FieldElement {
    fn from(data: &[u8]) -> Self {
        let mut value: [u8; 32] = [0; 32];
        value.copy_from_slice(data); 
        Self {
            value: value
        }
    }
}

impl From<u8> for FieldElement {
    fn from(data: u8) -> Self {
        let mut f = FieldElement::new();
        f.value[31] = data;
        f
    }
}

impl From<FieldElement> for u8 {
    fn from(fe: FieldElement) -> u8 {
        fe.value[31]
    }
}

// assumes usize < FieldElement (should be true, especially on the nano)
impl From<usize> for FieldElement {
    fn from(num: usize) -> Self {
        let mut f = FieldElement::new();
        let size_of_usize = core::mem::size_of::<usize>();
        let offset = if size_of_usize >= f.value.len() {
            0
        } else {
            f.value.len() - size_of_usize
        };

        for i in 0..size_of_usize {
            f.value[offset + i] = (num >> ((size_of_usize - 1 - i) * 8)) as u8;
        }

        f
    }
}

impl From<FieldElement> for usize {
    fn from(fe: FieldElement) -> usize {
        let mut value: usize = 0;
        let size_of_usize = core::mem::size_of::<usize>();
        let offset = if size_of_usize >= fe.value.len() {
            0
        } else {
            fe.value.len() - size_of_usize
        };

        for i in 0..size_of_usize {
            value |= (fe.value[i + offset] as usize) << ((size_of_usize - 1 - i) * 8);
        }

        value
    }
}

#[derive(Debug, Copy, Clone)]
pub struct CallV0 {
    pub to: FieldElement,
    pub entry_point_length: u8,
    pub entry_point: [u8; 32],
    pub selector: FieldElement,
    pub data_offset: FieldElement,
    pub data_len: FieldElement,
}

#[derive(Debug, Clone)]
pub struct CallV1 {
    pub to: FieldElement,
    pub selector: FieldElement,
    pub call_data: Vec<FieldElement, 32>
}

impl CallV0 {
    pub fn new() -> Self {
        Self {
            to: FieldElement::new(),
            entry_point_length: 0,
            entry_point: [0u8; 32],
            selector: FieldElement::new(),
            data_offset: FieldElement::new(),
            data_len: FieldElement::new() 
        }
    }

    pub fn clear(&mut self) {
        self.to.clear();
        self.entry_point_length = 0;
        self.entry_point.fill(0);
        self.selector.clear();
        self.data_offset.clear();
        self.data_len.clear();
    }
}

impl CallV1 {
    pub fn new() -> Self {
        Self {
            to: FieldElement::new(),
            selector: FieldElement::new(),
            call_data: Vec::new()
        }
    }

    pub fn clear(&mut self) {
        self.to.clear();
        self.selector.clear();
        self.call_data.clear();
    }
}

/// Maximum numbers of calls in a multicall Tx (out of memory)
/// NanoS = 3
/// NanoS+ = 10 (maybe more ?) 
const MAX_TX_CALLS: usize = 10;

pub struct CallDataV0 {
    pub call_array_len: FieldElement,
    pub calls: [CallV0; MAX_TX_CALLS],
    pub calldata_len: FieldElement,
} 

pub struct CallDataV1 {
    pub call_array_len: FieldElement,
    pub calls: Vec<CallV1, MAX_TX_CALLS>
}

impl CallDataV0 {
    pub fn new() -> Self {
        Self {
            call_array_len: FieldElement::new(),
            calls: [CallV0::new(); MAX_TX_CALLS],
            calldata_len: FieldElement::new()
        }
    }

    pub fn clear(&mut self) {
        self.call_array_len.clear();
        for i in 0..self.calls.len() {
            self.calls[i].clear();
        }
        self.calldata_len.clear();
    }
}

impl CallDataV1 {
    pub fn new() -> Self {
        Self {
            call_array_len: FieldElement::new(),
            calls: Vec::new()
        }
    }

    pub fn clear(&mut self) {
        self.call_array_len.clear();
        self.calls.clear();
    }
}

#[derive(Debug, Copy, Clone)]
pub enum AbstractCallData {
    Felt(FieldElement),
    Ref(usize),
    CallRef(usize, usize),
}

#[derive(Debug, Clone)]
pub struct AbstractCall {
    pub to: FieldElement,
    pub method: String<32>,
    pub selector: FieldElement,
    pub calldata: Vec<AbstractCallData, 16>
}

impl AbstractCall { 
    pub fn new() -> Self {
        Self {
            to: FieldElement::new(),
            method: String::new(),
            selector: FieldElement::new(),
            calldata: Vec::new()
        }
    }

    pub fn clear(&mut self) {
        self.to.clear();
        self.method.clear();
        self.selector.clear();
        self.calldata.clear();
    }
}

#[derive(Debug, Clone)]
pub struct Call {
    pub to: FieldElement,
    pub method: String<32>,
    pub selector: FieldElement,
    pub calldata: Vec<FieldElement, 8>
}

impl Call { 
    pub fn new() -> Self {
        Self {
            to: FieldElement::new(),
            method: String::new(),
            selector: FieldElement::new(),
            calldata: Vec::new()
        }
    }

    pub fn clear(&mut self) {
        self.to.clear();
        self.method.clear();
        self.selector.clear();
        self.calldata.clear();
    }
}

pub struct TransactionInfo {
    pub sender_address: FieldElement,
    pub max_fee: FieldElement,
    pub nonce: FieldElement,
    pub version: FieldElement,
    pub chain_id: FieldElement,
}

impl TransactionInfo {
    pub fn new() -> Self {
        Self {
            sender_address: FieldElement::new(),
            max_fee: FieldElement::new(),
            nonce: FieldElement::new(),
            version: FieldElement::new(),
            chain_id: FieldElement::new()
        }
    }

    pub fn clear(&mut self) {
        self.sender_address.clear();
        self.max_fee.clear();
        self.nonce.clear();
        self.version.clear();
        self.chain_id.clear();
    }
}

pub struct Transaction {
    pub tx_info: TransactionInfo,
    pub calldata: Vec<Call, MAX_TX_CALLS>
}

impl Transaction {
    pub fn new() -> Self {
        Self {
            tx_info: TransactionInfo::new(),
            calldata: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.tx_info.clear();
        self.calldata.clear();
    }
}