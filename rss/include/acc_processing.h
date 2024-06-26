// Copyright (c) Acconeer AB, 2020-2024
// All rights reserved

#ifndef ACC_PROCESSING_H_
#define ACC_PROCESSING_H_

#include <stdbool.h>
#include <stdint.h>

#include "acc_config_subsweep.h"
#include "acc_definitions_a121.h"
#include "acc_definitions_common.h"


/**
 * @defgroup processing Processing
 * @ingroup service
 *
 * @brief Module to interpret and process data read from sensor
 *
 * @{
 */


/**
 * @brief Generic processing handle
 */
struct acc_processing_handle;

typedef struct acc_processing_handle acc_processing_t;


/**
 * @brief Metadata that will be populated by the processing module during creation
 */
typedef struct
{
	/** Number of elements in the frame */
	uint16_t frame_data_length;
	/** Number of elements in the sweep */
	uint16_t sweep_data_length;
	/** Offset to the subsweeps data */
	uint16_t subsweep_data_offset[ACC_MAX_NUM_SUBSWEEPS];
	/** Number of elements in the subsweeps */
	uint16_t subsweep_data_length[ACC_MAX_NUM_SUBSWEEPS];
	/** Maximum sweep rate that the sensor can provide for the given configuration.
	 *  Note that this is not the actual exact sweep rate. To obtain an exact rate,
	 *  use the sweep rate parameter, @ref acc_config_sweep_rate_set.
	 *
	 *  If no max sweep rate is applicable, it's set to 0.0f.
	 */
	float max_sweep_rate;
	/** Flag indicating if high speed mode is used.
	 *  If true, it means that the sensor has been configured in a way where it
	 *  can optimize its measurements and obtain a high max_sweep_rate.
	 *
	 *  Configuration limitations to enable high speed mode:
	 *
	 *  continuous_sweep_mode false, see @ref acc_config_continuous_sweep_mode_set
	 *  inter_sweep_idle_state READY, see @ref acc_config_inter_sweep_idle_state_set
	 *  num_subsweeps 1, see @ref acc_config_num_subsweeps_set
	 *  profile 3-5, see @ref acc_config_profile_set
	 */
	bool high_speed_mode;
} acc_processing_metadata_t;


/**
 * @brief Result provided by the processing module
 */
typedef struct
{
	/** Indication of sensor data being saturated, can cause data corruption.
	 *  Lower the receiver gain if this indication is set.
	 */
	bool data_saturated;
	/** Indication of a delayed frame.
	 *  The frame rate might need to be lowered if this indication is set.
	 */
	bool frame_delayed;
	/** Indication of calibration needed
	 *  The sensor calibration needs to be redone if this indication is set.
	 */
	bool calibration_needed;
	/** Temperature in sensor during measurement (in degree Celsius).
	 *  Note that it has poor absolute accuracy and should only be used
	 *  for relative temperature measurements.
	 */
	int16_t temperature;
	/** Pointer to the frame data */
	acc_int16_complex_t *frame;
} acc_processing_result_t;


/**
 * @brief Create a processing instance with the provided configuration
 *
 * @param[in] config The configuration to create a processing instance with
 * @param[out] processing_metadata The metadata of the created processing instance
 * @return Processing handle, NULL if processing instance was not possible to create
 */
acc_processing_t *acc_processing_create(const acc_config_t *config, acc_processing_metadata_t *processing_metadata);


/**
 * @brief Process the data according to the configuration used in create
 *
 * @param[in] handle  A reference to the processing handle
 * @param[in] buffer  A reference to the buffer (populated by @ref acc_sensor_read) containing the
 *                    data to be processed.
 *
 * @param[out] result Processing result
 */
void acc_processing_execute(acc_processing_t *handle, void *buffer,
                            acc_processing_result_t *result);


/**
 * @brief Destroy a processing instance identified with the provided processing handle
 *
 * @param[in] handle A reference to the processing handle to destroy, can be NULL
 */
void acc_processing_destroy(acc_processing_t *handle);


/**
 * @brief Convert a distance or step length in points to meter
 *
 * Does not include any zero-point offset since it is highly integration dependant. In other words,
 * calling this function with a 0 always returns 0.0.
 *
 * @param[in] points Number of points to convert to meter
 * @return The corresponding length in meters
 */
float acc_processing_points_to_meter(int32_t points);


/**
 * @brief Convert a distance or step length in meter to points
 *
 * Does not include any zero-point offset since it is highly integration dependant. In other words,
 * calling this function with a 0.0 always returns 0.
 *
 * @param[in] length Length in meter to convert to points
 * @return The corresponding length in points
 */
int32_t acc_processing_meter_to_points(float length);


/**
 * @brief Calculate temperature compensation for mean sweep and background noise
 *        (tx off) standard deviation
 *
 * The signal adjustment models how the amplitude level fluctuates with temperature.
 * If the same object is measured against while the temperature changes,
 * the amplitude level should be multiplied with this factor.
 *
 * Example of usage:
 * int16_t reference_temperature (recorded temperature during calibration)
 * float reference_amplitude (recorded amplitude during calibration)
 *
 * int16_t current_temperature (temperature at current measurement time)
 *
 * float signal_adjust_factor
 * float deviation_adjust_factor
 *
 * acc_processing_get_temperature_adjustment_factors(reference_temperature, current_temperature, profile,
 *                                                   &signal_adjust_factor, &deviation_adjust_factor)
 *
 * reference_amplitude_new = reference_amplitude * signal_adjust_factor
 *
 * The reference_amplitude_new is an approximation of what the calibrated amplitude
 * would be at the new temperature.
 *
 * Eg. When the temperature falls 60 degrees, the amplitude (roughly) doubles.
 * This yields a signal_adjust_factor of (about) 2.
 *
 * The signal adjustment model follows 2 ^ -(temperature_diff / model_parameter), where
 * model_parameter reflects the temperature difference relative the reference temperature,
 * required for the amplitude to double/halve.
 *
 * The deviation_adjust_factor works the same way, but is applied to a measurement
 * taken with the Tx off. So instead of a measurement of amplitude, we have a measurement of tx_off.
 * The procedure for calculating this is to take the same configuration as
 * the application will use, but turning off the Tx.
 * This calibration value is multiplied with the deviation_adjust_factor.
 *
 * @param[in] reference_temperature The reference temperature, usually taken at calibration
 * @param[in] current_temperature The current temperature
 * @param[in] profile Configured profile
 * @param[out] signal_adjust_factor The calculated signal adjustment factor
 * @param[out] deviation_adjust_factor The calculated deviation adjustment factor
 */
void acc_processing_get_temperature_adjustment_factors(int16_t reference_temperature, int16_t current_temperature, acc_config_profile_t profile,
                                                       float *signal_adjust_factor, float *deviation_adjust_factor);


/**
 * @}
 */


#endif
