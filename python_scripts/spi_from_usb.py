import serial

def main():
	print("Hello from spi_from_usb!")

	ser = serial.Serial("COM6")

	while True:
		s = input().encode("utf-8").split()
		assert len(s) == 3
		rw = s[0]
		addr = int(s[1], 0)

		if rw == "r":
			num = int(s[2], 0)
			first_byte = 0x80 | addr
			ser.write(bytes([first_byte, num]))
			for i in range(num): print(ser.read())
		elif rw == "w":
			assert len(s[2]) == 2
			first_byte = addr
			data = bytes([first_byte]) + bytes.fromhex(s[2])
			ser.write(data)
		else:
			print("Invalid command.")

if __name__ == "__main__":
	main()
