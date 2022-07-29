/******************************************************************************
 * @file    ble_std.h
 * @author  MCD
 * @brief   BLE standard definitions
 ******************************************************************************
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

#ifndef BLE_STD_H__
#define BLE_STD_H__


/* HCI packet type */
#define HCI_COMMAND_PKT_TYPE             0x01
#define HCI_ACLDATA_PKT_TYPE             0x02
#define HCI_EVENT_PKT_TYPE               0x04

/* HCI packet header size */
#define HCI_COMMAND_HDR_SIZE             4
#define HCI_ACLDATA_HDR_SIZE             5
#define HCI_EVENT_HDR_SIZE               3

/* HCI parameters length */
#define HCI_COMMAND_MAX_PARAM_LEN        255
#define HCI_ACLDATA_MAX_DATA_LEN         251  /* HC_LE_Data_Packet_Length */
#define HCI_EVENT_MAX_PARAM_LEN          255

/* HCI packet maximum size */
#define HCI_COMMAND_PKT_MAX_SIZE \
            (HCI_COMMAND_HDR_SIZE + HCI_COMMAND_MAX_PARAM_LEN)
#define HCI_ACLDATA_PKT_MAX_SIZE \
            (HCI_ACLDATA_HDR_SIZE + HCI_ACLDATA_MAX_DATA_LEN)
#define HCI_EVENT_PKT_MAX_SIZE \
            (HCI_EVENT_HDR_SIZE   + HCI_EVENT_MAX_PARAM_LEN)

/* HCI Event codes */
                                                
/* HCI_DISCONNECTION_COMPLETE_EVENT code: */
#define HCI_DISCONNECTION_COMPLETE_EVT_CODE                    0x05 
  
/* HCI_ENCRYPTION_CHANGE_EVENT code: */
#define HCI_ENCRYPTION_CHANGE_EVT_CODE                         0x08   

/* HCI_READ_REMOTE_VERSION_INFORMATION_COMPLETE EVENT code: */
#define HCI_READ_REMOTE_VERSION_INFORMATION_COMPLETE_EVT_CODE  0x0C   

/* HCI_COMMAND_COMPLETE_EVENT code: */
   #define HCI_COMMAND_COMPLETE_EVT_CODE                       0x0E   
   
/* HCI_COMMAND_STATUS_EVENT code: */
   #define HCI_COMMAND_STATUS_EVT_CODE                         0x0F   
   
/* HCI_HARDWARE_ERROR_EVENT code: */
  #define HCI_HARDWARE_ERROR_EVT_CODE                          0x10   
  
/* HCI_NUMBER_OF_COMPLETED_PACKETS_EVENT code: */
   #define HCI_NUMBER_OF_COMPLETED_PACKETS_EVT_CODE            0x13   
   
/* HCI_DATA_BUFFER_OVERFLOW_EVENT code: */
   #define HCI_DATA_BUFFER_OVERFLOW_EVT_CODE                   0x1A   
   
/* HCI_ENCRYPTION_KEY_REFRESH_COMPLETE_EVENT code: */
#define HCI_ENCRYPTION_KEY_REFRESH_COMPLETE_EVT_CODE           0x30   
              
/* */                                
#define HCI_LE_META_EVT_CODE                                   0x3E 
  
/* */                  
#define HCI_VENDOR_SPECIFIC_DEBUG_EVT_CODE                     0xFF   

/* HCI LE SubEvent codes */

 /* HCI_LE_CONNECTION_COMPLETE_EVENT code: */
 #define HCI_LE_CONNECTION_COMPLETE_SUBEVT_CODE                 0x01
 
 /* HCI_LE_ADVERTISING_REPORT_EVENT code: */
 #define HCI_LE_ADVERTISING_REPORT_SUBEVT_CODE                  0x02
 
 /* HCI_LE_CONNECTION_UPDATE_COMPLETE_EVENT code: */
 #define HCI_LE_CONNECTION_UPDATE_COMPLETE_SUBEVT_CODE          0x03 
 
 /* HCI_LE_READ_REMOTE_FEATURES_COMPLETE_EVENT code: */
 #define HCI_LE_READ_REMOTE_FEATURES_COMPLETE_SUBEVT_CODE       0x04
 
 /* HCI_LE_LONG_TERM_KEY_REQUEST_EVENT code: */
 #define HCI_LE_LONG_TERM_KEY_REQUEST_SUBEVT_CODE               0x05
 
 /* HCI_LE_DATA_LENGTH_CHANGE_EVENT code: */
 #define HCI_LE_DATA_LENGTH_CHANGE_SUBEVT_CODE                  0x07 
 
 /* HCI_LE_READ_LOCAL_P256_PUBLIC_KEY_COMPLETE_EVENT code: */
 #define HCI_LE_READ_LOCAL_P256_PUBLIC_KEY_COMPLETE_SUBEVT_CODE 0x08
 
 /* HCI_LE_GENERATE_DHKEY_COMPLETE_EVENT code: */
 #define HCI_LE_GENERATE_DHKEY_COMPLETE_SUBEVT_CODE             0x09
 
 /* HCI_LE_ENHANCED_CONNECTION_COMPLETE_EVENT code: */
 #define HCI_LE_ENHANCED_CONNECTION_COMPLETE_SUBEVT_CODE        0x0A
 
 /* HCI_LE_DIRECT_ADVERTISING_REPORT_EVENT code: */
 #define HCI_LE_DIRECT_ADVERTISING_REPORT_SUBEVT_CODE           0x0B 
 
 /* HCI_LE_PHY_UPDATE_COMPLETE_EVENT code: */
 #define HCI_LE_PHY_UPDATE_COMPLETE_SUBEVT_CODE                 0x0C   

/* HCI error code */
#define HCI_SUCCESS_ERR_CODE                                   0x00
#define HCI_UNKNOWN_HCI_COMMAND_ERR_CODE                       0x01
#define HCI_UNKNOWN_CONNECTION_IDENTIFIER_ERR_CODE             0x02
#define HCI_AUTHENTICATION_FAILURE_ERR_CODE                    0x05
#define HCI_PIN_OR_KEY_MISSING_ERR_CODE                        0x06
#define HCI_MEMORY_CAPACITY_EXCEEDED_ERR_CODE                  0x07
#define HCI_CONNECTION_TIMEOUT_ERR_CODE                        0x08
#define HCI_COMMAND_DISALLOWED_ERR_CODE                        0x0C
#define HCI_UNSUPPORTED_FEATURE_OR_PARAMETER_VALUE_ERR_CODE    0x11
#define HCI_INVALID_HCI_COMMAND_PARAMETERS_ERR_CODE            0x12
#define HCI_REMOTE_USER_TERMINATED_CONNECTION_ERR_CODE         0x13
#define HCI_CONNECTION_TERMINATED_BY_LOCAL_HOST_ERR_CODE       0x16
#define HCI_LMP_FEATURE_ERR_CODE                               0x1A
#define HCI_INVALID_LL_PARAMETERS_ERR_CODE                     0x1E
#define HCI_UNSPECIFIED_ERROR_ERR_CODE                         0x1F
#define HCI_LL_RESPONSE_TIMEOUT_ERR_CODE                       0x22
#define HCI_LL_PROCEDURE_COLLISION_ERR_CODE                    0x23
#define HCI_LMP_PDU_NOT_ALLOWED_ERR_CODE                       0x24
#define HCI_INSTANT_PASSED_ERR_CODE                            0x28
#define HCI_DIFFERENT_TRANSACTION_COLLISION_ERR_CODE           0x2A
#define HCI_PARAMETER_OUT_OF_MANDATORY_RANGE_ERR_CODE          0x30
#define HCI_HOST_BUSY_PAIRING_ERR_CODE                         0x38
#define HCI_CONTROLLER_BUSY_ERR_CODE                           0x3A
#define HCI_ADVERTISING_TIMEOUT_ERR_CODE                       0x3C
#define HCI_CONNECTION_TERMINATED_DUE_TO_MIC_FAILURE_ERR_CODE  0x3D
#define HCI_CONNECTION_FAILED_TO_BE_ESTABLISHED_ERR_CODE       0x3E

/* HCI_LE_Read_PHY */
#define HCI_TX_PHY_LE_1M                 0x01
#define HCI_TX_PHY_LE_2M                 0x02
#define HCI_TX_PHY_LE_CODED              0x03
#define HCI_RX_PHY_LE_1M                 0x01
#define HCI_RX_PHY_LE_2M                 0x02
#define HCI_RX_PHY_LE_CODED              0x03

/* HCI_LE_Set_PHY */
#define HCI_ALL_PHYS_TX_NO_PREF          0x01
#define HCI_ALL_PHYS_RX_NO_PREF          0x02
#define HCI_TX_PHYS_LE_1M_PREF           0x01
#define HCI_TX_PHYS_LE_2M_PREF           0x02
#define HCI_TX_PHYS_LE_CODED_PREF        0x04
#define HCI_RX_PHYS_LE_1M_PREF           0x01
#define HCI_RX_PHYS_LE_2M_PREF           0x02
#define HCI_RX_PHYS_LE_CODED_PREF        0x04


#endif /* BLE_STD_H__ */

/*********************** (C) COPYRIGHT STMicroelectronics *****END OF FILE****/
