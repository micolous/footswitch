/*
 * footswitch/keyboard/keyboard.ino
 * USB HID Keyboard + Serial footswtich controller firmware.
 * https://github.com/micolous/footswitch
 *
 * Copyright 2021 Michael Farrell <micolous+git@gmail.com>
 * Based on Arduino's InputPullupSerial example, by Scott Fitzgerald.
 *
 * This Arduino code is released into the public domain, or can be used under
 * the Creative Commons Zero or Apache 2 licenses (your choice).
 *
 * Connect the switch between digital pin 2 and ground. When connected, this
 * will:
 *
 * - Press the F13 key
 * - Send a "1" via serial
 * - Turns on the LED (pin 13)
 *
 * When released, this will:
 *
 * - Release the F13 key
 * - Send a "0" via serial
 * - Turns off the LED (pin 13)
 *
 * On "Pro Micro" boards, this uses the RX LED (pin 17) instead.
 */
#include <Keyboard.h>

// ** CONFIGURATION PARAMETERS **

// Keycode to send, default is KEY_F13. List of supported codes:
// https://www.arduino.cc/reference/en/language/functions/usb/keyboard/keyboardmodifiers/
const char keyCode = KEY_F13;

// Input pin for the footswitch. Connect to the other side of the switch to
// ground.
const int buttonPin = 2;

// Output pin for the LED.
#ifdef ARDUINO_AVR_PROMICRO
// "Pro Micro" boards don't have an LED wired to the "usual" pin 13,
// use the RX LED (17) instead.
// https://learn.sparkfun.com/tutorials/pro-micro--fio-v3-hookup-guide/example-1-blinkies
const int ledPin = 17;
#else
// Use default LED pin, this is pin 13 on most boards.
const int ledPin = LED_BUILTIN;
#endif

// Debounce time for the input pin, in milliseconds.
// Increase if the input "flickers".
const unsigned long debounceDelay = 50;

// ** END CONFIGURATION PARAMETERS **

// The current state of the output pin.
int ledState = LOW;
// The current reading from the input pin.
int buttonState;
// The previous reading from the input pin.
int lastButtonState = LOW;
// The last time the output pin was toggled.
unsigned long lastDebounceTime = 0;

void setup() {
  // Configure input pin and enable the internal pull-up resistor
  pinMode(buttonPin, INPUT_PULLUP);
  pinMode(ledPin, OUTPUT);

  // Initialize keyboard device
  Keyboard.begin();

  // Start serial connection
  Serial.begin(9600);
}

void loop() {
  // Read the button state into a variable.
  int sensorVal = digitalRead(2);

  // check to see if you just pressed the button
  // (i.e. the input went from LOW to HIGH), and you've waited long enough
  // since the last press to ignore any noise:

  // If the switch changed, due to noise or pressing:
  if (sensorVal != lastButtonState) {
    // reset the debouncing timer
    lastDebounceTime = millis();
  }

  if ((millis() - lastDebounceTime) > debounceDelay) {
    // whatever the reading is at, it's been there for longer than the debounce
    // delay, so take it as the actual current state:

    // if the button state has changed:
    if (sensorVal != buttonState) {
      buttonState = sensorVal;

      // Keep in mind the pull-up means the pushbutton's logic is inverted. It goes
      // HIGH when it's open, and LOW when it's pressed.
      if (buttonState == LOW) {
        Keyboard.press(keyCode);
        Serial.print("1");
      } else {
        Keyboard.releaseAll();
        Serial.print("0");
      }
#ifdef ARDUINO_AVR_PROMICRO
      // Pro Micro has inverted LED state (LOW = on)
      ledState = buttonState == HIGH;
#else
      ledState = buttonState == LOW;
#endif
    }
  }

  // set the LED:
  digitalWrite(ledPin, ledState);

  // save the reading. Next time through the loop, it'll be the lastButtonState:
  lastButtonState = sensorVal;
}
