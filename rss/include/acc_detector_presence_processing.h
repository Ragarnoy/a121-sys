// Copyright (c) Acconeer AB, 2022-2024
// All rights reserved

#ifndef ACC_DETECTOR_PRESENCE_PROCESSING_H_
#define ACC_DETECTOR_PRESENCE_PROCESSING_H_

#include <stdbool.h>
#include <stdint.h>

#include "acc_config.h"
#include "acc_definitions_common.h"


/**
 * @brief Presence detector handle container
 */
struct acc_detector_presence_processing_handle;

typedef struct acc_detector_presence_processing_handle acc_detector_presence_processing_handle_t;


/**
 * @brief Presence processing configuration container
 */
struct  acc_detector_presence_processing_config;

typedef struct acc_detector_presence_processing_config acc_detector_presence_processing_config_t;


/**
 * @brief Presence detector results container
 */
typedef struct
{
	bool     presence_detected;
	float    intra_presence_score;
	float    inter_presence_score;
	float    presence_distance;
	float    *depthwise_intra_presence_scores;
	float    *depthwise_inter_presence_scores;
	uint32_t depthwise_presence_scores_length;
} acc_detector_presence_processing_result_t;


/**
 * @brief Get the buffer size needed for presence processing
 *
 * @param[in] sensor_config Sensor config to get size for
 * @return The buffer size
 */
uint32_t acc_detector_presence_processing_get_buffer_size(const acc_config_t *sensor_config);


/**
 * @brief Create a configuration for a presence processor
 *
 * @return Presence processor configuration, NULL if creation was not possible
 */
acc_detector_presence_processing_config_t *acc_detector_presence_processing_config_create(void);


/**
 * @brief Destroy a presence processor configuration
 *
 * @param[in] processor_config The configuration to destroy
 */
void acc_detector_presence_processing_config_destroy(acc_detector_presence_processing_config_t *processor_config);


/**
 * @brief Create a processing handle with the provided base configuration
 *
 * @param[in] processing_config The presence processing configuration to create a processing handle with
 * @param[in] sensor_config The sensor config to create a processing handle with
 * @return processing handle, NULL if creation was not possible
 */
acc_detector_presence_processing_handle_t *acc_detector_presence_processing_create(
	const acc_detector_presence_processing_config_t *processing_config,
	const acc_config_t                              *sensor_config);


/**
 * @brief Destroy a processing handle
 *
 * @param[in] processing_handle A reference to the processing handle to destroy
 */
void acc_detector_presence_processing_destroy(acc_detector_presence_processing_handle_t *processing_handle);


/**
 * @brief Reset existing processing buffers
 *
 * @param[in] processing_handle A reference to the processing handle to reset existing processing buffers for
 * @return true if successful, false otherwise
 */

bool acc_detector_presence_processing_reset(acc_detector_presence_processing_handle_t *processing_handle);


/**
 * @brief Process sensor data
 *
 * This function process the sensor data and tries to find presence.
 *
 * @param[in] processing_handle The presence detector processing handle to get the next result for
 * @param[in] buffer Working memory for process, must be at least 32-bit aligned
 * @param[in] frame The radar data to be processed, must be of the length provided at processing creation
 * @param[out] presence_result The result container with presence information for the user
 * @return true if successful, false otherwise
 */
bool acc_detector_presence_processing_process(acc_detector_presence_processing_handle_t *processing_handle,
                                              void                                      *buffer,
                                              const acc_int16_complex_t                 *frame,
                                              acc_detector_presence_processing_result_t *presence_result);


/**
 * @brief Set the frame rate
 *
 * See @ref acc_config_frame_rate_set for details
 *
 * @param[in] processing_config The configuration
 * @param[in] frame_rate Frame rate in Hz. Must be > 0
 */
void acc_detector_presence_processing_config_frame_rate_set(acc_detector_presence_processing_config_t *processing_config, float frame_rate);


/**
 * @brief Get the frame rate
 *
 * See @ref acc_detector_presence_config_frame_rate_set
 *
 * @param[in] processing_config The configuration
 * @return Frame rate in Hz
 */
float acc_detector_presence_processing_config_frame_rate_get(const acc_detector_presence_processing_config_t *processing_config);


/**
 * @brief Set the inter-frame presence timeout in seconds
 *
 * Number of seconds the inter-frame presence score needs to decrease before exponential
 * scaling starts for faster decline. Should be between 0 and 30 where 0 means no timeout
 *
 * @param[in] processing_config The configuration
 * @param[in] inter_frame_presence_timeout Timeout in seconds between 0 and 30
 */
void acc_detector_presence_processing_config_inter_frame_presence_timeout_set(acc_detector_presence_processing_config_t *processing_config,
                                                                              uint16_t                                  inter_frame_presence_timeout);


/**
 * @brief Get the inter-frame presence timeout in seconds
 *
 * See @ref acc_detector_presence_processing_config_inter_frame_presence_timeout_set
 *
 * @param[in] processing_config The configuration
 * @return Inter-frame presence timeout in s
 */
uint16_t acc_detector_presence_processing_config_inter_frame_presence_timeout_get(const acc_detector_presence_processing_config_t *processing_config);


/**
 * @brief Set inter-frame phase boost
 *
 * Used to increase detection of slow motions by utilizing the phase information in the Sparse IQ data.
 *
 * @param[in] processing_config The configuration to set inter phase boost for
 * @param[in] enable true if inter phase boost should be enabled
 */
void acc_detector_presence_processing_config_inter_phase_boost_set(acc_detector_presence_processing_config_t *processing_config, bool enable);


/**
 * @brief Get if inter-frame phase boost is enabled
 *
 * See @ref acc_detector_presence_processing_config_inter_phase_boost_set
 *
 * @param[in] processing_config The configuration to get inter phase boost for
 * @return true if inter-frame phase boost is enabled, false otherwise
 */
bool acc_detector_presence_processing_config_inter_phase_boost_get(const acc_detector_presence_processing_config_t *processing_config);


/**
 * @brief Set intra-frame presence detection
 *
 * This is used for detecting faster movements inside frames
 *
 * @param[in] processing_config The configuration to set intra-frame detection for
 * @param[in] enable true if intra-frame detection should be enabled
 */
void acc_detector_presence_processing_config_intra_detection_set(acc_detector_presence_processing_config_t *processing_config, bool enable);


/**
 * @brief Get if frame intra-frame presence detection is enabled
 *
 * See @ref acc_detector_presence_processing_config_intra_detection_set
 *
 * @param[in] processing_config The configuration to get intra detection for
 * @return true if intra-frame detection is enabled, false otherwise
 */
bool acc_detector_presence_processing_config_intra_detection_get(const acc_detector_presence_processing_config_t *processing_config);


/**
 * @brief Set the detection threshold for the intra-frame presence detection
 *
 * This is the threshold for detecting faster movements inside frames
 *
 * @param[in] processing_config The configuration to set the detection threshold for
 * @param[in] intra_detection_threshold The intra-frame detection threshold to set
 */
void acc_detector_presence_processing_config_intra_detection_threshold_set(acc_detector_presence_processing_config_t *processing_config,
                                                                           float                                     intra_detection_threshold);


/**
 * @brief Get the detection threshold for the intra-frame presence detection
 *
 * See @ref acc_detector_presence_processing_config_intra_detection_threshold_set
 *
 * @param[in] processing_config The configuration to get the detection threshold for
 * @return The intra-frame detection threshold
 */
float acc_detector_presence_processing_config_intra_detection_threshold_get(const acc_detector_presence_processing_config_t *processing_config);


/**
 * @brief Set inter-frame presence detection
 *
 * This is used for detecting slower movements between frames
 *
 * @param[in] processing_config The configuration to set inter-frame detection for
 * @param[in] enable true if inter-frame presence detection should be enabled
 */
void acc_detector_presence_processing_config_inter_detection_set(acc_detector_presence_processing_config_t *processing_config, bool enable);


/**
 * @brief Get if inter-frame presence detection is enabled
 *
 * See @ref acc_detector_presence_processing_config_inter_detection_set
 *
 * @param[in] processing_config The configuration to get inter-frame presence detection for
 * @return true if inter-frame presence detection is enabled, false otherwise
 */
bool acc_detector_presence_processing_config_inter_detection_get(const acc_detector_presence_processing_config_t *processing_config);


/**
 * @brief Set the detection threshold for the inter-frame presence detection
 *
 * This is the threshold for detecting slower movements between frames
 *
 * @param[in] processing_config The configuration to set the detection threshold for
 * @param[in] inter_detection_threshold The threshold
 */
void acc_detector_presence_processing_config_inter_detection_threshold_set(acc_detector_presence_processing_config_t *processing_config,
                                                                           float                                     inter_detection_threshold);


/**
 * @brief Get the detection threshold for the inter-frame presence detection
 *
 * See @ref acc_detector_presence_processing_config_inter_detection_threshold_set
 *
 * @param[in] processing_config The configuration to get the detection threshold for
 * @return detection threshold
 */
float acc_detector_presence_processing_config_inter_detection_threshold_get(const acc_detector_presence_processing_config_t *processing_config);


/**
 * @brief Set the time constant of the low pass filter for the inter-frame deviation between fast and slow
 *
 * @param[in] processing_config The configuration
 * @param[in] inter_frame_deviation_time_const Time constant to set
 */
void acc_detector_presence_processing_config_inter_frame_deviation_time_const_set(acc_detector_presence_processing_config_t *processing_config,
                                                                                  float                                     inter_frame_deviation_time_const);


/**
 * @brief Get the time constant of the low pass filter for the inter-frame deviation between fast and slow
 *
 * @param[in] processing_config The configuration to get the time constant for
 * @return time constant in s
 */
float acc_detector_presence_processing_config_inter_frame_deviation_time_const_get(
	const acc_detector_presence_processing_config_t *processing_config);


/**
 * @brief Set the cutoff frequency of the low pass filter for the fast filtered absolute sweep mean
 *
 * No filtering is applied if the cutoff is set over half the frame rate (Nyquist limit).
 *
 * @param[in] processing_config The configuration
 * @param[in] inter_frame_fast_cutoff Cutoff frequency to set
 */
void acc_detector_presence_processing_config_inter_frame_fast_cutoff_set(acc_detector_presence_processing_config_t *processing_config,
                                                                         float                                     inter_frame_fast_cutoff);


/**
 * @brief Get the cutoff frequency of the low pass filter for the fast filtered absolute sweep mean
 *
 * @param[in] processing_config The configuration to get the cutoff frequency for
 * @return the cutoff frequency in Hz
 */
float acc_detector_presence_processing_config_inter_frame_fast_cutoff_get(const acc_detector_presence_processing_config_t *processing_config);


/**
 * @brief Set the cutoff frequency of the low pass filter for the slow filtered absolute sweep mean
 *
 * @param[in] processing_config The configuration
 * @param[in] inter_frame_slow_cutoff Cutoff frequency to set
 */
void acc_detector_presence_processing_config_inter_frame_slow_cutoff_set(acc_detector_presence_processing_config_t *processing_config,
                                                                         float                                     inter_frame_slow_cutoff);


/**
 * @brief Get the cutoff frequency of the low pass filter for the slow filtered absolute sweep mean
 *
 * @param[in] processing_config The configuration to get the cutoff frequency for
 * @return the cutoff frequency in Hz
 */
float acc_detector_presence_processing_config_inter_frame_slow_cutoff_get(const acc_detector_presence_processing_config_t *processing_config);


/**
 * @brief Set the time constant for the depthwise filtering in the intra-frame part
 *
 * @param[in] processing_config The configuration
 * @param[in] intra_frame_time_const Time constant to set
 */
void acc_detector_presence_processing_config_intra_frame_time_const_set(acc_detector_presence_processing_config_t *processing_config,
                                                                        float                                     intra_frame_time_const);


/**
 * @brief Get the time constant for the depthwise filtering in the intra-frame part
 *
 * @param[in] processing_config The configuration to get the time constant for
 * @return time constant in s
 */
float acc_detector_presence_processing_config_intra_frame_time_const_get(const acc_detector_presence_processing_config_t *processing_config);


/**
 * @brief Set the time constant for the output in the intra-frame part
 *
 * @param[in] processing_config The configuration
 * @param[in] intra_output_time_const Time constant to set
 */
void acc_detector_presence_processing_config_intra_output_time_const_set(acc_detector_presence_processing_config_t *processing_config,
                                                                         float                                     intra_output_time_const);


/**
 * @brief Get the time constant for the output in the intra-frame part
 *
 * @param[in] processing_config The configuration to get the time constant for
 * @return time constant in s
 */
float acc_detector_presence_processing_config_intra_output_time_const_get(const acc_detector_presence_processing_config_t *processing_config);


/**
 * @brief Set the time constant for the output in the inter-frame part
 *
 * @param[in] processing_config The configuration
 * @param[in] inter_output_time_const Time constant to set
 */
void acc_detector_presence_processing_config_inter_output_time_const_set(acc_detector_presence_processing_config_t *processing_config,
                                                                         float                                     inter_output_time_const);


/**
 * @brief Get the time constant for the output in the inter-frame part
 *
 * @param[in] processing_config The configuration to get the time constant for
 * @return time constant in s
 */
float acc_detector_presence_processing_config_inter_output_time_const_get(const acc_detector_presence_processing_config_t *processing_config);


#endif
