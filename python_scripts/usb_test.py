import serial

def main():
	print("Hello from usb_test!")

	ser = serial.Serial("COM6")

	while True:
		s = input().encode("utf-8")
		ser.write(s)
		for i in range(len(s)): print(ser.read())



if __name__ == "__main__":
	main()
