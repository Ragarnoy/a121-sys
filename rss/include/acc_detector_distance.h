// Copyright (c) Acconeer AB, 2022-2025
// All rights reserved

#ifndef ACC_DETECTOR_DISTANCE_H_
#define ACC_DETECTOR_DISTANCE_H_

#include <stdbool.h>
#include <stdint.h>

#include "acc_definitions_a121.h"
#include "acc_definitions_common.h"
#include "acc_detector_distance_definitions.h"
#include "acc_processing.h"
#include "acc_sensor.h"

/**
 * @defgroup Distance Distance Detector
 *
 * @brief Distance Detector API description
 *
 * For a thorough description of the Distance Detector, see the
 * Distance Detector User Guide.
 * @{
 */

/**
 * @brief The maximum number of estimated distances that is returned in the Detector result
 */
#define ACC_DETECTOR_DISTANCE_RESULT_MAX_NUM_DISTANCES (10U)

/**
 * @brief Distance Detector handle
 *
 * Used when calling the Distance Detector control and process functions.
 */
typedef struct acc_detector_distance_handle acc_detector_distance_handle_t;

/**
 * @brief Distance Detector configuration
 *
 * Used when calling the Distance Detector configuration functions.
 */
typedef struct acc_detector_distance_config acc_detector_distance_config_t;

/**
 * @brief Distance Detector result returned from @ref acc_detector_distance_process
 */
typedef struct
{
	/**
	 * Estimated distances (m).
	 *
	 * Sorted according to the selected peak sorting strategy.
	 */
	float distances[ACC_DETECTOR_DISTANCE_RESULT_MAX_NUM_DISTANCES];

	/**
	 * Estimated reflector strengths (dB).
	 *
	 * Corresponds to the peak amplitude of the estimated distances.
	 */
	float strengths[ACC_DETECTOR_DISTANCE_RESULT_MAX_NUM_DISTANCES];

	/**
	 * The number of valid distances in the distances array.
	 *
	 * Corresponds to the number of detected objects according to the threshold method.
	 * If 0, no objects were detected.
	 */
	uint8_t num_distances;

	/**
	 * Indication of an object close to the start edge, located outside of the measurement range.
	 */
	bool near_start_edge_status;

	/**
	 * Indication of calibration needed.
	 *
	 * Is set if the temperature has changed by at least 15 degrees Celsius from
	 * the temperature at calibration.
	 *
	 * If true:
	 *  - The sensor calibration needs to be redone by calling @ref acc_sensor_calibrate
	 *  - The Detector calibration needs to be updated by calling @ref acc_detector_distance_update_calibration
	 */
	bool calibration_needed;

	/**
	 * Temperature in sensor during measurement (in degree Celsius).
	 *
	 * Note that it has poor absolute accuracy and should only be used
	 * for relative temperature measurements.
	 */
	int16_t temperature;

	/**
	 * Processing result, including IQ data, that the Distance detection is based on.
	 *
	 * This will point to memory in the buffer supplied to @ref acc_detector_distance_process
	 *
	 * Note: The processing result is only valid until the next time
	 *       @ref acc_detector_distance_process is called.
	 */
	acc_processing_result_t *processing_result;

	/**
	 * Metadata for the processing result
	 *
	 * Note: The processing metedata is only valid until the next time
	 *        @ref acc_detector_distance_process is called.
	 */
	acc_processing_metadata_t *processing_metadata;

	/**
	 * The sensor_config used for the processing result
	 *
	 * Note: The sensor_config is only valid until the next time
	 *       @ref acc_detector_distance_process is called.
	 */
	const acc_config_t *sensor_config;
} acc_detector_distance_result_t;

/**
 * @brief Create a Distance Detector configuration
 *
 * The returned configuration is used when calling the Distance configuration functions.
 *
 * @return Distance Detector configuration, NULL at failure
 */
acc_detector_distance_config_t *acc_detector_distance_config_create(void);

/**
 * @brief Destroy Distance Detector configuration
 *
 * To be called when Distance Detector configuration is no longer needed,
 * typically at end of application.
 *
 * @param[in] config The Distance Detector configuration to destroy
 */
void acc_detector_distance_config_destroy(acc_detector_distance_config_t *config);

/**
 * @brief Log Distance configuration, typically through printf
 *
 * If handle != NULL, the underlying sensor configs will also be logged.
 *
 * @param[in] handle The Distance Detector handle, can be NULL
 * @param[in] config The Distance Detector configuration
 */
void acc_detector_distance_config_log(const acc_detector_distance_handle_t *handle, const acc_detector_distance_config_t *config);

/**
 * @brief Get the memory size needed to use the Distance Detector API functions
 *
 * buffer_size is the size of working memory (buffer) needed for the different Detector functions.
 * This includes memory for sensor control and Detector calculations.
 * This memory can be reused between sensor instances and Detector handles.
 *
 * detector_cal_result_static_size is the size of the static part of the Detector calibration result.
 *
 * Both sizes are dependent on the configuration used which is contained in the provided handle.
 *
 * @param[in] handle The Distance Detector handle
 * @param[out] buffer_size The buffer size
 * @param[out] detector_cal_result_static_size The calibration result size
 * @return true if successful, false otherwise
 */
bool acc_detector_distance_get_sizes(const acc_detector_distance_handle_t *handle, uint32_t *buffer_size, uint32_t *detector_cal_result_static_size);

/**
 * @brief Create Distance Detector handle using the provided Distance Detector configuration
 *
 * The returned handle is used when calling the Detector control and process functions.
 *
 * @param[in] config The Distance Detector configuration
 * @return Distance Detector handle, NULL at failure
 */
acc_detector_distance_handle_t *acc_detector_distance_create(const acc_detector_distance_config_t *config);

/**
 * @brief Destroy Distance Detector handle, freeing its resources
 *
 * To be called when Distance Detector handle is no longer needed,
 * typically at end of application.
 *
 * @param[in] handle The Distance Detector handle to destroy
 */
void acc_detector_distance_destroy(acc_detector_distance_handle_t *handle);

/**
 * @brief Calibrate detector
 *
 * Calibrating the Detector is needed to ensure reliable Detector results.
 *
 * See the calibration sections in the Distance Detector User Guide for more information.
 *
 * @param[in] sensor The sensor instance to use for calibration
 * @param[in] handle The Distance Detector handle
 * @param[in] sensor_cal_result Sensor calibration result
 * @param[in] buffer Working memory buffer needed by function
 * @param[in] buffer_size The size of buffer. Needs to be at least
 *            the result of @ref acc_detector_distance_get_sizes
 * @param[out] detector_cal_result_static Static result of calibration
 * @param[in] detector_cal_result_static_size The size of detector_cal_result_static.
 *            Needs to be at least the result of @ref acc_detector_distance_get_sizes
 * @param[out] detector_cal_result_dynamic Dynamic result of calibration
 * @param[out] calibration_complete Will be set to true when the calibration is complete.
 *             If false; at least one more call to this function is needed.
 *             Note that it's necessary to wait for interrupt between calls.
 * @return true if successful, false otherwise
 */
bool acc_detector_distance_calibrate(acc_sensor_t                      *sensor,
                                     acc_detector_distance_handle_t    *handle,
                                     const acc_cal_result_t            *sensor_cal_result,
                                     void                              *buffer,
                                     uint32_t                           buffer_size,
                                     uint8_t                           *detector_cal_result_static,
                                     uint32_t                           detector_cal_result_static_size,
                                     acc_detector_cal_result_dynamic_t *detector_cal_result_dynamic,
                                     bool                              *calibration_complete);

/**
 * @brief Perform a subset of the full Detector calibration
 *
 * This function should be called if the @ref acc_detector_distance_result_t.calibration_needed indication is set.
 * A new sensor calibration needs to be done before calling this function.
 *
 * See the calibration sections in the Distance Detector User Guide for more information.
 *
 * @param[in] sensor The sensor instance to use for calibration update
 * @param[in] handle The Detector handle
 * @param[in] sensor_cal_result Sensor calibration result
 * @param[in] buffer Working memory buffer needed by function
 * @param[in] buffer_size The size of buffer. Needs to be at least
 *            the result of @ref acc_detector_distance_get_sizes
 * @param[out] detector_cal_result_dynamic Result of the calibration update
 * @param[out] calibration_complete Will be set to true when the calibration update is complete.
 *             If false; at least one more call to this function is needed.
 *             Note that it's necessary to wait for interrupt between calls.
 * @return true if successful, false otherwise
 */
bool acc_detector_distance_update_calibration(acc_sensor_t                      *sensor,
                                              acc_detector_distance_handle_t    *handle,
                                              const acc_cal_result_t            *sensor_cal_result,
                                              void                              *buffer,
                                              uint32_t                           buffer_size,
                                              acc_detector_cal_result_dynamic_t *detector_cal_result_dynamic,
                                              bool                              *calibration_complete);

/**
 * @brief Prepare the Detector for measurements
 *
 * This function loads the provided configuration and calibration result
 * to the sensor via SPI.
 *
 * This should be done before every measure.
 *
 * @param[in, out] handle The Distance Detector handle
 * @param[in] config The Distance Detector configuration
 * @param[in] sensor The sensor instance
 * @param[in] sensor_cal_result The sensor calibration result
 * @param[in] buffer Memory used by the detector. Should be at least buffer_size bytes
 * @param[in] buffer_size The buffer size received by @ref acc_detector_distance_get_sizes
 * @return true if successful, false otherwise
 */
bool acc_detector_distance_prepare(const acc_detector_distance_handle_t *handle,
                                   const acc_detector_distance_config_t *config,
                                   acc_sensor_t                         *sensor,
                                   const acc_cal_result_t               *sensor_cal_result,
                                   void                                 *buffer,
                                   uint32_t                              buffer_size);

/**
 * @brief Process sensor data into a Detector result
 *
 * This function takes sensor data (an IQ data frame) as input and finds distances
 * to objects. For more information on the processing, see the Distance Detector
 * User Guide.
 *
 * Needs to be called multiple times until 'result_available' is true.
 * See example_detector_distance.c how this is done.
 *
 * @param[in] handle The Distance Detector handle
 * @param[in] buffer A reference to the buffer (populated by @ref acc_sensor_read) containing the
 *                   data to be processed.
 * @param[in] detector_cal_result_static The result from @ref acc_detector_distance_calibrate
 * @param[in] detector_cal_result_dynamic The result from @ref acc_detector_distance_calibrate or @ref acc_detector_distance_update_calibration
 * @param[out] result_available True if the result  Whether result will contain a new result
 * @param[out] result Distance Detector result
 * @return true if successful, false otherwise
 */
bool acc_detector_distance_process(acc_detector_distance_handle_t    *handle,
                                   void                              *buffer,
                                   uint8_t                           *detector_cal_result_static,
                                   acc_detector_cal_result_dynamic_t *detector_cal_result_dynamic,
                                   bool                              *result_available,
                                   acc_detector_distance_result_t    *result);

/**
 * @brief Set start of measured interval (m)
 *
 * This should be used to make sure that objects in the desired range
 * interval can be detected.
 *
 * @param[out] config The Distance Detector configuration
 * @param[in] start_m The start of measured interval (m)
 */
void acc_detector_distance_config_start_set(acc_detector_distance_config_t *config, float start_m);

/**
 * @brief Get start of measured interval (m)
 *
 * See @ref acc_detector_distance_config_start_set.
 *
 * @param[in] config The Distance Detector configuration
 * @return The start of measured interval (m)
 */
float acc_detector_distance_config_start_get(const acc_detector_distance_config_t *config);

/**
 * @brief Set end of measured interval (m)
 *
 * This should be used to make sure that objects in the desired range
 * interval can be detected.
 *
 * For example, if objects in the environment can appear between
 * 0.5 m and 2.0 m from the sensor, an end of around 2.1 m is probably a good setting.
 *
 * @param[out] config The Distance Detector configuration
 * @param[in] end_m The end of measured interval
 */
void acc_detector_distance_config_end_set(acc_detector_distance_config_t *config, float end_m);

/**
 * @brief Get end of measured interval (m)
 *
 * See @ref acc_detector_distance_config_end_set.
 *
 * @param[in] config The Distance Detector configuration
 * @return The end of measured interval
 */
float acc_detector_distance_config_end_get(const acc_detector_distance_config_t *config);

/**
 * @brief Set maximum step length
 *
 * Step length is the number of 2.5 mm distance points between each
 * measurement point in the underlying IQ data. This is normally
 * automatically set by the detector depending on the range start/end
 * configurations.
 *
 * This function sets an upper limit to the automatically set step length.
 * If set to 0 (default), no upper limit is enforced.
 *
 * A small maximum step length increases SNR (signal quality) through more
 * efficient distance filtering, but also increases the measurement time
 * and power consumption.
 *
 * @param[out] config The Distance Detector configuration
 * @param[in] max_step_length The maximum step length
 */
void acc_detector_distance_config_max_step_length_set(acc_detector_distance_config_t *config, uint16_t max_step_length);

/**
 * @brief Get maximum step length
 *
 * See @ref acc_detector_distance_config_max_step_length_set.
 *
 * @param[in] config The Distance Detector configuration
 * @return The maximum step length
 */
uint16_t acc_detector_distance_config_max_step_length_get(const acc_detector_distance_config_t *config);

/**
 * @brief Enable close range leakage cancellation
 *
 * Close range leakage cancellation refers to the process of measuring close to the
 * sensor(<100mm) by first characterizing the direct leakage, and then subtracting it
 * from the measured sweep in order to isolate the signal component of interest.
 *
 * The close range leakage cancellation process requires the sensor to be installed in its
 * intended geometry with free space in front of the sensor during Detector calibration.
 *
 * This is default off.
 *
 * @param[out] config The Distance Detector configuration
 * @param[in] enable true to enable close range leakage cancellation logic, false to disable
 */
void acc_detector_distance_config_close_range_leakage_cancellation_set(acc_detector_distance_config_t *config, bool enable);

/**
 * @brief Get if close range leakage cancellation is enabled
 *
 * See @ref acc_detector_distance_config_close_range_leakage_cancellation_set.
 *
 * @param[in] config The Distance Detector configuration
 * @return true if enabled, false if disabled
 */
bool acc_detector_distance_config_close_range_leakage_cancellation_get(const acc_detector_distance_config_t *config);

/**
 * @brief Set signal quality (dB)
 *
 * High signal quality results in better SNR (signal-to-noise ratio).
 * High signal quality leads to longer measurement time which leads to higher power consumption.
 * Signal quality can be set within the interval [-10, 35].
 *
 * @param[out] config The Distance Detector configuration
 * @param[in] signal_quality The signal quality
 */
void acc_detector_distance_config_signal_quality_set(acc_detector_distance_config_t *config, float signal_quality);

/**
 * @brief Get signal quality
 *
 * See @ref acc_detector_distance_config_signal_quality_set.
 *
 * @param[in] config The Distance Detector configuration
 * @return The signal quality
 */
float acc_detector_distance_config_signal_quality_get(const acc_detector_distance_config_t *config);

/**
 * @brief Set maximum Profile
 *
 * Profile sets the duration and shape of the emitted pulse from the sensor.
 * This is normally automatically set by the detector depending on the range
 * start/end configurations.
 *
 * This function sets an upper limit to the automatically set Profile.
 *
 * A low maximum profile improves the radial distance resolution but decreases the SNR.
 *
 * @param[out] config The Distance Detector configuration
 * @param[in] max_profile The maximum profile
 */
void acc_detector_distance_config_max_profile_set(acc_detector_distance_config_t *config, acc_config_profile_t max_profile);

/**
 * @brief Get maximum Profile
 *
 * See @ref acc_detector_distance_config_max_profile_set.
 *
 * @param[in] config The Distance Detector configuration
 * @return The maximum Profile
 */
acc_config_profile_t acc_detector_distance_config_max_profile_get(const acc_detector_distance_config_t *config);

/**
 * @brief Set threshold method
 *
 * Sets which method to use for determining if a peak in the underlying
 * radar data is a reflection or not (and will be returned as a distance estimation).
 *
 * See @ref acc_detector_distance_threshold_method_t for a short description
 * of each method.
 *
 * More detailed information on thresholds can be found in the Distance Detector
 * User Guide.
 *
 * @param[out] config The Distance Detector configuration
 * @param[in] threshold_method The threshold method
 */
void acc_detector_distance_config_threshold_method_set(acc_detector_distance_config_t          *config,
                                                       acc_detector_distance_threshold_method_t threshold_method);

/**
 * @brief Get threshold method
 *
 * See @ref acc_detector_distance_config_threshold_method_set.
 *
 * @param[in] config The Distance Detector configuration
 * @return The threshold method
 */
acc_detector_distance_threshold_method_t acc_detector_distance_config_threshold_method_get(const acc_detector_distance_config_t *config);

/**
 * @brief Set peak sorting method
 *
 * Sets the method used determining the order of the resulting
 * estimated distances.
 *
 * See @ref acc_detector_distance_peak_sorting_t for a short description
 * of each method.
 *
 * More detailed information on peak sorting can be found in the Distance Detector
 * User Guide.
 *
 * @param[out] config The Distance Detector configuration
 * @param[in] peak_sorting The peak sorting method
 */
void acc_detector_distance_config_peak_sorting_set(acc_detector_distance_config_t *config, acc_detector_distance_peak_sorting_t peak_sorting);

/**
 * @brief Get peak sorting method
 *
 * See @ref acc_detector_distance_config_peak_sorting_set.
 *
 * @param[in] config The Distance Detector configuration
 * @return The peak sorting method
 */
acc_detector_distance_peak_sorting_t acc_detector_distance_config_peak_sorting_get(const acc_detector_distance_config_t *config);

/**
 * @brief Set number of frames to use for recorded threshold
 *
 * If recorded threshold is used, the Distance Detector calibration collects
 * IQ data to create a threshold. The number of frames sets how much IQ data
 * is collected.
 *
 * A lower number reduces calibration time and a higher number results
 * in a more statistically significant threshold.
 *
 * See @ref acc_detector_distance_config_threshold_method_set for more information.
 *
 * @param[out] config The Distance Detector configuration
 * @param[in] num_frames The number of frames
 */
void acc_detector_distance_config_num_frames_recorded_threshold_set(acc_detector_distance_config_t *config, uint16_t num_frames);

/**
 * @brief Get number of frames to use for recorded threshold
 *
 * See @ref acc_detector_distance_config_num_frames_recorded_threshold_set.
 *
 * @param[in] config The Distance Detector configuration
 * @return The number of frames
 */
uint16_t acc_detector_distance_config_num_frames_recorded_threshold_get(const acc_detector_distance_config_t *config);

/**
 * @brief Set fixed amplitude threshold value
 *
 * See @ref acc_detector_distance_config_threshold_method_set for more information.
 *
 * @param[out] config The Distance Detector configuration
 * @param[in] fixed_threshold_value The fixed threshold value
 */
void acc_detector_distance_config_fixed_amplitude_threshold_value_set(acc_detector_distance_config_t *config, float fixed_threshold_value);

/**
 * @brief Get fixed amplitude threshold value
 *
 * See @ref acc_detector_distance_config_fixed_amplitude_threshold_value_set.
 *
 * @param[in] config The Distance Detector configuration
 * @return The fixed threshold value
 */
float acc_detector_distance_config_fixed_amplitude_threshold_value_get(const acc_detector_distance_config_t *config);

/**
 * @brief Set fixed strength threshold value
 *
 * See @ref acc_detector_distance_config_threshold_method_set for more information.
 *
 * @param[out] config The Distance Detector configuration
 * @param[in] fixed_threshold_value The fixed threshold value
 */
void acc_detector_distance_config_fixed_strength_threshold_value_set(acc_detector_distance_config_t *config, float fixed_threshold_value);

/**
 * @brief Get fixed strength threshold value
 *
 * See @ref acc_detector_distance_config_fixed_strength_threshold_value_set.
 *
 * @param[in] config The Distance Detector configuration
 * @return The fixed threshold value
 */
float acc_detector_distance_config_fixed_strength_threshold_value_get(const acc_detector_distance_config_t *config);

/**
 * @brief Set threshold sensitivity
 *
 * Set how sensitive the threshold is. Only applicable for
 * @ref ACC_DETECTOR_DISTANCE_THRESHOLD_METHOD_RECORDED and
 * @ref ACC_DETECTOR_DISTANCE_THRESHOLD_METHOD_CFAR.
 *
 * A high sensitivity will make the Distance Detector find more reflections,
 * but there is a higher risk that some of the reflection are false positives.
 *
 * Threshold sensitivity can be set within the interval [0, 1].
 *
 * @param[out] config The Distance Detector configuration
 * @param[in] threshold_sensitivity The threshold sensitivity
 */
void acc_detector_distance_config_threshold_sensitivity_set(acc_detector_distance_config_t *config, float threshold_sensitivity);

/**
 * @brief Get threshold sensitivity
 *
 * See @ref acc_detector_distance_config_threshold_sensitivity_set.
 *
 * @param[in] config The Distance Detector configuration
 * @return The threshold sensitivity
 */
float acc_detector_distance_config_threshold_sensitivity_get(const acc_detector_distance_config_t *config);

/**
 * @brief Set reflector shape
 *
 * Set the expected shape of objects to be found by the Distance Detector.
 * This is used to optimize underlying sensor settings.
 *
 * See the Distance Detector User Guide for more information.
 *
 * @param[out] config The Distance Detector configuration
 * @param[in] reflector_shape The reflector shape
 */
void acc_detector_distance_config_reflector_shape_set(acc_detector_distance_config_t         *config,
                                                      acc_detector_distance_reflector_shape_t reflector_shape);

/**
 * @brief Get reflector shape
 *
 * See @ref acc_detector_distance_config_reflector_shape_set.
 *
 * @param[in] config The Distance Detector configuration
 * @return The reflector shape
 */
acc_detector_distance_reflector_shape_t acc_detector_distance_config_reflector_shape_get(const acc_detector_distance_config_t *config);

/**
 * @}
 */

#endif
