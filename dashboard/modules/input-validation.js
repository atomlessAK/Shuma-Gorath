// @ts-check

const IPV4_SEGMENT_PATTERN = /^\d{1,3}$/;
const IPV6_INPUT_PATTERN = /^[0-9a-fA-F:.]+$/;

const sanitizeIntegerText = (value) => (value || '').replace(/[^\d]/g, '');
const sanitizeIpText = (value) => (value || '').replace(/[^0-9a-fA-F:.]/g, '');

const isValidIpv4 = (value) => {
  const parts = value.split('.');
  if (parts.length !== 4) return false;
  return parts.every((part) => {
    if (!IPV4_SEGMENT_PATTERN.test(part)) return false;
    if (part.length > 1 && part.startsWith('0')) return false;
    const num = Number.parseInt(part, 10);
    return num >= 0 && num <= 255;
  });
};

const isValidIpv6 = (value) => {
  if (!IPV6_INPUT_PATTERN.test(value)) return false;
  try {
    new URL(`http://[${value}]/`);
    return true;
  } catch (_e) {
    return false;
  }
};

const isValidIpAddress = (value) => {
  if (!value) return false;
  if (value.includes(':')) return isValidIpv6(value);
  if (value.includes('.')) return isValidIpv4(value);
  return false;
};

export const create = (options = {}) => {
  const getById = typeof options.getById === 'function' ? options.getById : () => null;
  const setFieldError = typeof options.setFieldError === 'function' ? options.setFieldError : () => {};
  const integerFieldRules = options.integerFieldRules || {};
  const banDurationBoundsSeconds = options.banDurationBoundsSeconds || { min: 60, max: 31536000 };
  const banDurationFields = options.banDurationFields || {};
  const manualBanDurationField = options.manualBanDurationField || null;
  const onFieldInteraction =
    typeof options.onFieldInteraction === 'function' ? options.onFieldInteraction : () => {};

  const parseIntegerLoose = (id) => {
    const input = getById(id);
    const rules = integerFieldRules[id];
    if (!input || !rules) return null;
    const sanitized = sanitizeIntegerText(input.value);
    if (input.value !== sanitized) input.value = sanitized;
    if (sanitized.length === 0) return null;
    const parsed = Number.parseInt(sanitized, 10);
    if (!Number.isInteger(parsed)) return null;
    return parsed;
  };

  const validateIntegerFieldById = (id, showInline = false) => {
    const input = getById(id);
    const rules = integerFieldRules[id];
    if (!input || !rules) return false;
    const parsed = parseIntegerLoose(id);
    if (parsed === null) {
      setFieldError(input, `${rules.label} is required.`, showInline);
      return false;
    }
    if (parsed < rules.min || parsed > rules.max) {
      setFieldError(input, `${rules.label} must be between ${rules.min} and ${rules.max}.`, showInline);
      return false;
    }
    setFieldError(input, '', showInline);
    return true;
  };

  const readIntegerFieldValue = (id, _messageTarget) => {
    const input = getById(id);
    const rules = integerFieldRules[id];
    if (!input || !rules) return null;
    if (!validateIntegerFieldById(id, true)) {
      const parsed = parseIntegerLoose(id);
      const message = parsed === null
        ? `${rules.label} is required.`
        : `${rules.label} must be between ${rules.min} and ${rules.max}.`;
      void message;
      input.reportValidity();
      input.focus();
      return null;
    }
    const value = parseIntegerLoose(id);
    input.value = String(value);
    setFieldError(input, '', true);
    return value;
  };

  const durationPartsToSeconds = (days, hours, minutes) =>
    (days * 86400) + (hours * 3600) + (minutes * 60);

  const secondsToDurationParts = (totalSeconds, fallbackSeconds) => {
    const fallback = Number.parseInt(fallbackSeconds, 10) || 0;
    let seconds = Number.parseInt(totalSeconds, 10);
    if (!Number.isFinite(seconds) || seconds <= 0) seconds = fallback;
    if (seconds < banDurationBoundsSeconds.min) seconds = banDurationBoundsSeconds.min;
    if (seconds > banDurationBoundsSeconds.max) seconds = banDurationBoundsSeconds.max;
    return {
      days: Math.floor(seconds / 86400),
      hours: Math.floor((seconds % 86400) / 3600),
      minutes: Math.floor((seconds % 3600) / 60)
    };
  };

  const setDurationInputsFromSeconds = (group, totalSeconds) => {
    if (!group) return;
    const daysInput = getById(group.daysId);
    const hoursInput = getById(group.hoursId);
    const minutesInput = getById(group.minutesId);
    if (!daysInput || !hoursInput || !minutesInput) return;

    const parts = secondsToDurationParts(totalSeconds, group.fallback);
    daysInput.value = String(parts.days);
    hoursInput.value = String(parts.hours);
    minutesInput.value = String(parts.minutes);
  };

  const setBanDurationInputFromSeconds = (durationKey, totalSeconds) => {
    const group = banDurationFields[durationKey];
    setDurationInputsFromSeconds(group, totalSeconds);
  };

  const readDurationFromInputs = (group, showInline = false) => {
    if (!group) return null;

    const daysInput = getById(group.daysId);
    const hoursInput = getById(group.hoursId);
    const minutesInput = getById(group.minutesId);
    if (!daysInput || !hoursInput || !minutesInput) return null;

    const daysValid = validateIntegerFieldById(group.daysId, showInline);
    const hoursValid = validateIntegerFieldById(group.hoursId, showInline);
    const minutesValid = validateIntegerFieldById(group.minutesId, showInline);
    const days = parseIntegerLoose(group.daysId);
    const hours = parseIntegerLoose(group.hoursId);
    const minutes = parseIntegerLoose(group.minutesId);

    if (!daysValid || !hoursValid || !minutesValid || days === null || hours === null || minutes === null) {
      return null;
    }

    const totalSeconds = durationPartsToSeconds(days, hours, minutes);
    if (totalSeconds < banDurationBoundsSeconds.min || totalSeconds > banDurationBoundsSeconds.max) {
      const message = `${group.label} must be between 1 minute and 365 days.`;
      setFieldError(daysInput, message, showInline);
      setFieldError(hoursInput, message, showInline);
      setFieldError(minutesInput, message, showInline);
      return null;
    }

    setFieldError(daysInput, '', showInline);
    setFieldError(hoursInput, '', showInline);
    setFieldError(minutesInput, '', showInline);
    return { days, hours, minutes, totalSeconds };
  };

  const readBanDurationFromInputs = (durationKey, showInline = false) => {
    const group = banDurationFields[durationKey];
    return readDurationFromInputs(group, showInline);
  };

  const focusFirstInvalidDurationInput = (group) => {
    if (!group) return;
    const daysInput = getById(group.daysId);
    const hoursInput = getById(group.hoursId);
    const minutesInput = getById(group.minutesId);
    if (daysInput && !daysInput.checkValidity()) {
      daysInput.reportValidity();
      daysInput.focus();
      return;
    }
    if (hoursInput && !hoursInput.checkValidity()) {
      hoursInput.reportValidity();
      hoursInput.focus();
      return;
    }
    if (minutesInput && !minutesInput.checkValidity()) {
      minutesInput.reportValidity();
      minutesInput.focus();
      return;
    }
    if (daysInput) {
      daysInput.reportValidity();
      daysInput.focus();
    }
  };

  const readBanDurationSeconds = (durationKey) => {
    const group = banDurationFields[durationKey];
    if (!group) return null;
    const result = readDurationFromInputs(group, true);
    if (result) return result.totalSeconds;
    focusFirstInvalidDurationInput(group);
    return null;
  };

  const readManualBanDurationFromInputs = (showInline = false) =>
    readDurationFromInputs(manualBanDurationField, showInline);

  const readManualBanDurationSeconds = (_showInline = false) => {
    const result = readManualBanDurationFromInputs(true);
    if (result) return result.totalSeconds;
    focusFirstInvalidDurationInput(manualBanDurationField);
    return null;
  };

  const validateIpFieldById = (id, required, label, showInline = false) => {
    const input = getById(id);
    if (!input) return false;
    const sanitized = sanitizeIpText(input.value.trim());
    if (input.value !== sanitized) input.value = sanitized;

    if (!sanitized) {
      if (!required) {
        setFieldError(input, '', showInline);
        return true;
      }
      setFieldError(input, `${label} is required.`, showInline);
      return false;
    }

    if (!isValidIpAddress(sanitized)) {
      setFieldError(input, `${label} must be a valid IPv4 or IPv6 address.`, showInline);
      return false;
    }
    setFieldError(input, '', showInline);
    return true;
  };

  const readIpFieldValue = (id, required, _messageTarget, label) => {
    const input = getById(id);
    if (!input) return null;
    if (!validateIpFieldById(id, required, label, true)) {
      input.reportValidity();
      input.focus();
      return null;
    }
    const sanitized = sanitizeIpText(input.value.trim());
    input.value = sanitized;
    setFieldError(input, '', true);
    return sanitized;
  };

  const bindIntegerFieldValidation = (id) => {
    const input = getById(id);
    const rules = integerFieldRules[id];
    if (!input || !rules) return;

    const apply = (showInline = false) => {
      const sanitized = sanitizeIntegerText(input.value);
      if (input.value !== sanitized) input.value = sanitized;
      if (!sanitized) {
        setFieldError(input, `${rules.label} is required.`, showInline);
        return;
      }
      const parsed = Number.parseInt(sanitized, 10);
      if (!Number.isInteger(parsed)) {
        setFieldError(input, `${rules.label} must be a whole number.`, showInline);
        return;
      }
      if (parsed < rules.min || parsed > rules.max) {
        setFieldError(input, `${rules.label} must be between ${rules.min} and ${rules.max}.`, showInline);
        return;
      }
      setFieldError(input, '', showInline);
    };

    input.addEventListener('input', () => {
      apply(true);
      onFieldInteraction(id);
    });
    input.addEventListener('blur', () => {
      if (!input.value) {
        input.value = String(rules.fallback);
      }
      const parsed = parseIntegerLoose(id);
      if (parsed !== null && parsed < rules.min) input.value = String(rules.min);
      if (parsed !== null && parsed > rules.max) input.value = String(rules.max);
      apply(true);
      onFieldInteraction(id);
    });
    apply(false);
  };

  const bindIpFieldValidation = (id, required, label) => {
    const input = getById(id);
    if (!input) return;
    const apply = (showInline = false) => {
      validateIpFieldById(id, required, label, showInline);
    };
    input.addEventListener('input', () => {
      apply(true);
      onFieldInteraction(id);
    });
    input.addEventListener('blur', () => {
      apply(true);
      onFieldInteraction(id);
    });
    apply(false);
  };

  return {
    parseIntegerLoose,
    validateIntegerFieldById,
    readIntegerFieldValue,
    validateIpFieldById,
    readIpFieldValue,
    setBanDurationInputFromSeconds,
    readBanDurationFromInputs,
    readBanDurationSeconds,
    readManualBanDurationFromInputs,
    readManualBanDurationSeconds,
    bindIntegerFieldValidation,
    bindIpFieldValidation
  };
};
