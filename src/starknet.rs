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

#[derive(Debug, Copy, Clone)]
pub struct CallArray {
    pub to: FieldElement,
    pub entry_point_length: u8,
    pub entry_point: [u8; 32],
    pub selector: FieldElement,
    pub data_offset: FieldElement,
    pub data_len: FieldElement,
}

#[derive(Debug, Copy, Clone)]
pub struct CallArrayV1 {
    pub to: FieldElement,
    pub selector: FieldElement,
    pub call_data_len: FieldElement,
    pub call_data: [FieldElement; 6]
}

impl CallArray {
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

impl CallArrayV1 {
    pub fn new() -> Self {
        Self {
            to: FieldElement::new(),
            selector: FieldElement::new(),
            call_data_len: FieldElement::new(),
            call_data: [FieldElement::ZERO; 6]
        }
    }

    pub fn clear(&mut self) {
        self.to.clear();
        self.selector.clear();
        self.call_data_len.clear();
        for i in 0..6 {
            self.call_data[i].clear();
        }
    }
}

/// Maximum numbers of calls in a multicall Tx (out of memory)
/// NanoS = 3
/// NanoS+ = 10 (maybe more ?) 
const MAX_TX_CALLS: usize = 3;

pub struct CallData {
    pub call_array_len: FieldElement,
    pub calls: [CallArray; MAX_TX_CALLS],
    pub calldata_len: FieldElement,
} 

pub struct CallDataV1 {
    pub call_array_len: FieldElement,
    pub calls: [CallArrayV1; MAX_TX_CALLS]
}

impl CallData {
    pub fn new() -> Self {
        Self {
            call_array_len: FieldElement::new(),
            calls: [CallArray::new(); MAX_TX_CALLS],
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
            calls: [CallArrayV1::new(); MAX_TX_CALLS],
        }
    }

    pub fn clear(&mut self) {
        self.call_array_len.clear();
        for i in 0..self.calls.len() {
            self.calls[i].clear();
        }
    }
}

pub struct Transaction {
    pub sender_address: FieldElement,
    pub calldata: CallData,             
    pub calldata_v1: CallDataV1,
    pub max_fee: FieldElement,
    pub nonce: FieldElement,
    pub version: FieldElement,
    pub chain_id: FieldElement
}

impl Transaction {
    pub fn new() -> Self {
        Self {
            sender_address: FieldElement::new(),
            calldata: CallData::new(),
            calldata_v1: CallDataV1::new(),
            max_fee: FieldElement::new(),
            nonce: FieldElement::new(),
            version: FieldElement::new(),
            chain_id: FieldElement::new()
        }
    }

    pub fn clear(&mut self) {
        self.sender_address.clear();
        self.calldata.clear();
        self.calldata_v1.clear();
        self.max_fee.clear();
        self.nonce.clear();
        self.version.clear();
        self.chain_id.clear();
    }
}