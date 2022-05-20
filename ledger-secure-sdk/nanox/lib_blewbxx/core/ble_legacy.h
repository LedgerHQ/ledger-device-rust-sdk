/*****************************************************************************
 * @file    ble_legacy.h
 * @author  MCD
 * @brief   This file contains legacy definitions used for BLE.
 *****************************************************************************
 * @attention
 *
 * <h2><center>&copy; Copyright (c) 2021 STMicroelectronics.
 * All rights reserved.</center></h2>
 *
 * This software component is licensed by ST under Ultimate Liberty license
 * SLA0044, the "License"; You may not use this file except in compliance with
 * the License. You may obtain a copy of the License at:
 *                             www.st.com/SLA0044
 *
 ******************************************************************************
 */

#ifndef BLE_LEGACY_H__
#define BLE_LEGACY_H__


/* ------------------------------------------------------------------------- */


/*
 * The event code in the @ref hci_event_pckt structure.
 * If event code is HCI_VENDOR_SPECIFIC_DEBUG_EVT_CODE,  application can use @ref evt_blecore_aci
 * structure to parse the packet.
 */
   
#define EVT_VENDOR                      HCI_VENDOR_SPECIFIC_DEBUG_EVT_CODE               /* 0xFF */

#define EVT_CONN_COMPLETE               HCI_LE_CONNECTION_UPDATE_COMPLETE_SUBEVT_CODE    /* 0x03 */
#define EVT_DISCONN_COMPLETE            HCI_DISCONNECTION_COMPLETE_EVT_CODE              /* 0x05 */
#define EVT_LE_META_EVENT               HCI_LE_META_EVT_CODE                             /* 0x3E */
#define EVT_LE_CONN_UPDATE_COMPLETE     HCI_LE_CONNECTION_UPDATE_COMPLETE_SUBEVT_CODE    /* 0x03 */
#define EVT_LE_CONN_COMPLETE            HCI_LE_CONNECTION_COMPLETE_SUBEVT_CODE           /* 0x01 */
#define EVT_LE_ADVERTISING_REPORT       HCI_LE_ADVERTISING_REPORT_SUBEVT_CODE            /* 0x02 */
#define EVT_LE_PHY_UPDATE_COMPLETE      HCI_LE_PHY_UPDATE_COMPLETE_SUBEVT_CODE           /* 0x0C */
#define EVT_LE_ENHANCED_CONN_COMPLETE   HCI_LE_ENHANCED_CONNECTION_COMPLETE_SUBEVT_CODE  /* 0x0A */

typedef PACKED(struct) _hci_uart_pckt
{
  uint8_t type;
  uint8_t data[1];
} hci_uart_pckt;

typedef PACKED(struct) _hci_event_pckt
{
  uint8_t         evt;
  uint8_t         plen;
  uint8_t         data[1];
} hci_event_pckt;

typedef PACKED(struct) _evt_le_meta_event
{
  uint8_t         subevent;
  uint8_t         data[1];
} evt_le_meta_event;

/**
 * Vendor specific event for BLE core.
 */
typedef PACKED(struct) _evt_blecore_aci
{
  uint16_t ecode; /**< One of the BLE core event codes. */
  uint8_t  data[1];
} evt_blecore_aci;

#define evt_blue_aci evt_blecore_aci 


/* BLE core event codes */
#define EVT_BLUE_GATT_ATTRIBUTE_MODIFIED          ACI_GATT_ATTRIBUTE_MODIFIED_VSEVT_CODE          /*(0x0C01)*/
#define EVT_BLUE_GATT_PROCEDURE_TIMEOUT           ACI_GATT_PROC_TIMEOUT_VSEVT_CODE                /*(0x0C02)*/
#define EVT_BLUE_ATT_EXCHANGE_MTU_RESP            ACI_ATT_EXCHANGE_MTU_RESP_VSEVT_CODE            /*(0x0C03)*/
#define EVT_BLUE_ATT_FIND_INFORMATION_RESP        ACI_ATT_FIND_INFO_RESP_VSEVT_CODE               /*(0x0C04)*/
#define EVT_BLUE_ATT_FIND_BY_TYPE_VAL_RESP        ACI_ATT_FIND_BY_TYPE_VALUE_RESP_VSEVT_CODE      /*(0x0C05)*/
#define EVT_BLUE_ATT_READ_BY_TYPE_RESP            ACI_ATT_READ_BY_TYPE_RESP_VSEVT_CODE            /*(0x0C06)*/
#define EVT_BLUE_ATT_READ_RESP                    ACI_ATT_READ_RESP_VSEVT_CODE                    /*(0x0C07)*/
#define EVT_BLUE_ATT_READ_BLOB_RESP               ACI_ATT_READ_BLOB_RESP_VSEVT_CODE               /*(0x0C08)*/
#define EVT_BLUE_ATT_READ_MULTIPLE_RESP           ACI_ATT_READ_MULTIPLE_RESP_VSEVT_CODE           /*(0x0C09)*/
#define EVT_BLUE_ATT_READ_BY_GROUP_TYPE_RESP      ACI_ATT_READ_BY_GROUP_TYPE_RESP_VSEVT_CODE      /*(0x0C0A)*/
#define EVT_BLUE_ATT_PREPARE_WRITE_RESP           ACI_ATT_PREPARE_WRITE_RESP_VSEVT_CODE           /*(0x0C0C)*/
#define EVT_BLUE_ATT_EXEC_WRITE_RESP              ACI_ATT_EXEC_WRITE_RESP_VSEVT_CODE              /*(0x0C0D)*/
#define EVT_BLUE_GATT_INDICATION                  ACI_GATT_INDICATION_VSEVT_CODE                  /*(0x0C0E)*/
#define EVT_BLUE_GATT_NOTIFICATION                ACI_GATT_NOTIFICATION_VSEVT_CODE                /*(0x0C0F)*/
#define EVT_BLUE_GATT_PROCEDURE_COMPLETE          ACI_GATT_PROC_COMPLETE_VSEVT_CODE               /*(0x0C10)*/
#define EVT_BLUE_GATT_ERROR_RESP                  ACI_GATT_ERROR_RESP_VSEVT_CODE                  /*(0x0C11)*/
#define EVT_BLUE_GATT_DISC_READ_CHAR_BY_UUID_RESP ACI_GATT_DISC_READ_CHAR_BY_UUID_RESP_VSEVT_CODE /*(0x0C12)*/
#define EVT_BLUE_GATT_WRITE_PERMIT_REQ            ACI_GATT_WRITE_PERMIT_REQ_VSEVT_CODE            /*(0x0C13)*/
#define EVT_BLUE_GATT_READ_PERMIT_REQ             ACI_GATT_READ_PERMIT_REQ_VSEVT_CODE             /*(0x0C14)*/
#define EVT_BLUE_GATT_READ_MULTI_PERMIT_REQ       ACI_GATT_READ_MULTI_PERMIT_REQ_VSEVT_CODE       /*(0x0C15)*/
#define EVT_BLUE_GATT_TX_POOL_AVAILABLE           ACI_GATT_TX_POOL_AVAILABLE_VSEVT_CODE           /*(0x0C16)*/
#define EVT_BLUE_GATT_SERVER_CONFIRMATION_EVENT   ACI_GATT_SERVER_CONFIRMATION_VSEVT_CODE         /*(0x0C17)*/
#define EVT_BLUE_GATT_PREPARE_WRITE_PERMIT_REQ    ACI_GATT_PREPARE_WRITE_PERMIT_REQ_VSEVT_CODE    /*(0x0C18)*/

#define EVT_BLUE_GAP_LIMITED_DISCOVERABLE         ACI_GAP_LIMITED_DISCOVERABLE_VSEVT_CODE         /*(0x0400)*/
#define EVT_BLUE_GAP_PAIRING_CMPLT                ACI_GAP_PAIRING_COMPLETE_VSEVT_CODE             /*(0x0401)*/
#define EVT_BLUE_GAP_PASS_KEY_REQUEST             ACI_GAP_PASS_KEY_REQ_VSEVT_CODE                 /*(0x0402)*/
#define EVT_BLUE_GAP_AUTHORIZATION_REQUEST        ACI_GAP_AUTHORIZATION_REQ_VSEVT_CODE            /*(0x0403)*/
#define EVT_BLUE_GAP_SLAVE_SECURITY_INITIATED     ACI_GAP_SLAVE_SECURITY_INITIATED_VSEVT_CODE     /*(0X0404)*/
#define EVT_BLUE_GAP_BOND_LOST                    ACI_GAP_BOND_LOST_VSEVT_CODE                    /*(0X0405)*/

#define EVT_BLUE_GAP_DEVICE_FOUND                (0x0406)                                          /*removed*/                        

#define EVT_BLUE_GAP_PROCEDURE_COMPLETE           ACI_GAP_PROC_COMPLETE_VSEVT_CODE                /*(0x0407)*/
#define EVT_BLUE_GAP_ADDR_NOT_RESOLVED            ACI_GAP_ADDR_NOT_RESOLVED_VSEVT_CODE            /*(0x0408)*/
#define EVT_BLUE_GAP_NUMERIC_COMPARISON_VALUE     ACI_GAP_NUMERIC_COMPARISON_VALUE_VSEVT_CODE     /*(0x0409)*/
#define EVT_BLUE_GAP_KEYPRESS_NOTIFICATION        ACI_GAP_KEYPRESS_NOTIFICATION_VSEVT_CODE        /*(0x040A)*/

#define EVT_BLUE_L2CAP_CONNECTION_UPDATE_REQ      ACI_L2CAP_CONNECTION_UPDATE_REQ_VSEVT_CODE      /*(0x0802)*/
#define EVT_BLUE_L2CAP_CONNECTION_UPDATE_RESP     ACI_L2CAP_CONNECTION_UPDATE_RESP_VSEVT_CODE     /*(0x0800)*/


/* Macro to get RSSI from advertising report #0.
 * "p" must be a pointer to the event parameters buffer
 */
#define HCI_LE_ADVERTISING_REPORT_RSSI_0(p) \
        (*(int8_t*)((&((hci_le_advertising_report_event_rp0*)(p))-> \
                      Advertising_Report[0].Length_Data) + 1 + \
                    ((hci_le_advertising_report_event_rp0*)(p))-> \
                    Advertising_Report[0].Length_Data))


/* ------------------------------------------------------------------------- */


/* Bluetooth 48 bit address (in little-endian order).
 */
typedef	uint8_t	tBDAddr[6];


/* ------------------------------------------------------------------------- */


/* Error Codes as specified by the specification 
 */
#define ERR_CMD_SUCCESS                              HCI_SUCCESS_ERR_CODE                                    /*0x00*/
#define ERR_UNKNOWN_HCI_COMMAND	                     HCI_UNKNOWN_HCI_COMMAND_ERR_CODE                        /*0x01*/
#define ERR_UNKNOWN_CONN_IDENTIFIER                  HCI_UNKNOWN_CONNECTION_IDENTIFIER_ERR_CODE              /*0x02*/
#define ERR_AUTH_FAILURE                             HCI_AUTHENTICATION_FAILURE_ERR_CODE                     /*0x05*/
#define ERR_PIN_OR_KEY_MISSING                       HCI_PIN_OR_KEY_MISSING_ERR_CODE                         /*0x06*/
#define ERR_MEM_CAPACITY_EXCEEDED                    HCI_MEMORY_CAPACITY_EXCEEDED_ERR_CODE                   /*0x07*/
#define ERR_CONNECTION_TIMEOUT                       HCI_CONNECTION_TIMEOUT_ERR_CODE                         /*0x08*/
#define ERR_COMMAND_DISALLOWED                       HCI_COMMAND_DISALLOWED_ERR_CODE                         /*0x0C*/
#define ERR_UNSUPPORTED_FEATURE                      HCI_UNSUPPORTED_FEATURE_OR_PARAMETER_VALUE_ERR_CODE     /*0x11*/
#define ERR_INVALID_HCI_CMD_PARAMS                   HCI_INVALID_HCI_COMMAND_PARAMETERS_ERR_CODE             /*0x12*/
#define ERR_RMT_USR_TERM_CONN                        HCI_REMOTE_USER_TERMINATED_CONNECTION_ERR_CODE          /*0x13*/
#define ERR_RMT_DEV_TERM_CONN_LOW_RESRCES            0x14
#define ERR_RMT_DEV_TERM_CONN_POWER_OFF              0x15
#define ERR_LOCAL_HOST_TERM_CONN                     HCI_CONNECTION_TERMINATED_BY_LOCAL_HOST_ERR_CODE        /*0x16*/
#define ERR_UNSUPP_RMT_FEATURE                       HCI_LMP_FEATURE_ERR_CODE                                /*0x1A*/
#define ERR_INVALID_LMP_PARAM                        HCI_INVALID_LL_PARAMETERS_ERR_CODE                      /*0x1E*/
#define ERR_UNSPECIFIED_ERROR                        HCI_UNSPECIFIED_ERROR_ERR_CODE                          /*0x1F*/
#define ERR_LL_RESP_TIMEOUT                          HCI_LL_RESPONSE_TIMEOUT_ERR_CODE                        /*0x22*/
#define ERR_LMP_PDU_NOT_ALLOWED                      HCI_LMP_PDU_NOT_ALLOWED_ERR_CODE                        /*0x24*/
#define ERR_INSTANT_PASSED                           HCI_INSTANT_PASSED_ERR_CODE                             /*0x28*/
#define ERR_PAIR_UNIT_KEY_NOT_SUPP                   0x29
#define ERR_CONTROLLER_BUSY                          HCI_DIFFERENT_TRANSACTION_COLLISION_ERR_CODE            /*0x3A*/
#define ERR_DIRECTED_ADV_TIMEOUT                     HCI_ADVERTISING_TIMEOUT_ERR_CODE                        /*0x3C*/
#define ERR_CONN_END_WITH_MIC_FAILURE                HCI_CONNECTION_TERMINATED_DUE_TO_MIC_FAILURE_ERR_CODE   /*0x3D*/
#define ERR_CONN_FAILED_TO_ESTABLISH                 HCI_CONNECTION_FAILED_TO_BE_ESTABLISHED_ERR_CODE        /*0x3E*/

/* ------------------------------------------------------------------------- */


/* Bluetooth address types
 */
#define PUBLIC_ADDR                            0X00U
#define RANDOM_ADDR                            0X01U
#define STATIC_RANDOM_ADDR                     0X01U
#define RESOLVABLE_PRIVATE_ADDR                0X02U
#define NON_RESOLVABLE_PRIVATE_ADDR            0X03U


/* Scan_types Scan types
 */
#define PASSIVE_SCAN                           0x00U
#define ACTIVE_SCAN                            0x01U


/* ------------------------------------------------------------------------- */


/* Various obsolete definitions
 */

#define LIM_DISC_MODE_TIMEOUT                      180000 /* 180 seconds */
#define PRIVATE_ADDR_INT_TIMEOUT                   900000 /* 15 minutes */

#define BLE_STATUS_SEC_UNKNOWN_CONNECTION_ID         0x40
#define BLE_STATUS_INVALID_LEN_PDU                   0x44
#define FLASH_READ_FAILED                            0x49
#define FLASH_WRITE_FAILED                           0x4A
#define FLASH_ERASE_FAILED                           0x4B
#define TIMER_NOT_VALID_LAYER                        0x54
#define TIMER_INSUFFICIENT_RESOURCES                 0x55
#define BLE_STATUS_DEV_NOT_FOUND_IN_DB               0x5C
#define BLE_STATUS_INVALID_PARAMETER                 0x61
#define BLE_INSUFFICIENT_ENC_KEYSIZE                 0x65
#define BLE_STATUS_ADDR_NOT_RESOLVED                 0x70
#define BLE_STATUS_PROFILE_ALREADY_INITIALIZED       0xF0
#define BLE_STATUS_NULL_PARAM                        0xF1

#define ATT_MTU                                        23

#define DEVICE_NAME_LEN                                 7

#define CONFIG_DATA_DIV_OFFSET                       0x06
#define CONFIG_DATA_DIV_LEN                             2

#define USE_FIXED_PIN_FOR_PAIRING                    0x00
#define DONOT_USE_FIXED_PIN_FOR_PAIRING              0x01

#define SM_LINK_AUTHENTICATED                        0x01
#define SM_LINK_AUTHORIZED                           0x02
#define SM_LINK_ENCRYPTED                            0x04

#define SM_PAIRING_SUCCESS                           SMP_PAIRING_STATUS_SUCCESS         /*0x00*/
#define SM_PAIRING_TIMEOUT                           SMP_PAIRING_STATUS_SMP_TIMEOUT     /*0x01*/
#define SM_PAIRING_FAILED                            SMP_PAIRING_STATUS_PAIRING_FAILED  /*0x02*/

#define PASSKEY_ENTRY_FAILED                         0x01

#define ADV_IND                                      GAP_ADV_IND                   /*0*/
#define ADV_DIRECT_IND                               1
#define ADV_SCAN_IND                                 GAP_ADV_SCAN_IND              /*2*/
#define ADV_NONCONN_IND                              GAP_ADV_NONCONN_IND           /*3*/
#define SCAN_RSP                                     4
#define HIGH_DUTY_CYCLE_DIRECTED_ADV                 GAP_ADV_HIGH_DC_DIRECT_IND    /*1*/
#define LOW_DUTY_CYCLE_DIRECTED_ADV                  GAP_ADV_LOW_DC_DIRECT_IND     /*4*/

#define ADV_INTERVAL_LOWEST_CONN                     0X0020
#define ADV_INTERVAL_HIGHEST                         0X4000
#define ADV_INTERVAL_LOWEST_NONCONN                  0X00A0


/* ------------------------------------------------------------------------- */


/*
 * BLE_DEFAULT_MAX_ATT_MTU: maximum supported ATT MTU size.
 */
#define BLE_DEFAULT_MAX_ATT_MTU             158

/*
 * BLE_DEFAULT_MBLOCKS_COUNT: default memory blocks count
 */
#define BLE_DEFAULT_MBLOCKS_COUNT(n_link) \
          BLE_MBLOCKS_CALC(BLE_DEFAULT_PREP_WRITE_LIST_SIZE, \
                           BLE_DEFAULT_MAX_ATT_MTU, n_link)


#define TOTAL_DEVICE_ID_DATA_SIZE 56


/* ------------------------------------------------------------------------- */


/* Deprecative name for LE Read Remote Features command
 */
#define hci_le_read_remote_used_features hci_le_read_remote_features
#define hci_le_read_remote_used_features_complete_event_rp0 \
          hci_le_read_remote_features_complete_event_rp0


/* ------------------------------------------------------------------------- */


/* Byte order conversions
*/
#define htob( d, n )  (d)     /* LE */
#define btoh( d, n )  (d)     /* LE */


/* Events table
 */
#define HCI_LE_META_EVENT_TABLE_SIZE HCI_LE_EVENT_TABLE_SIZE
#define HCI_VENDOR_SPECIFIC_EVENT_TABLE_SIZE HCI_VS_EVENT_TABLE_SIZE
#define hci_le_meta_event_table hci_le_event_table
#define hci_vendor_specific_event_table hci_vs_event_table


/* ------------------------------------------------------------------------- */


#endif /* BLE_LEGACY_H__ */
