import serial

def print_line(ser):
	print("debug: print_line")
	head = ser.read(1)
	if head == b'E':
		data = []
		while ser.in_waiting > 0:
			read = ser.read(1)
			if read == b'\n':
				break
			data.append(read)
		print(b"".join(data).decode("utf-8"))
	elif head == b'O':
		while ser.in_waiting > 0:
			data = ser.read(1)
			print(data.hex(), end=" ")
			if data == b'\n':
				break
		print()
	print("debug: print_line done")

def main():
	print("Hello from spi_from_usb!")

	ser = serial.Serial("COM6")

	while True:
		s = input().split()
		assert len(s) == 3
		rw = s[0]
		addr = int(s[1], 0)
		# print(f"debug: rw={rw}, addr={addr}, data={s[2]}")

		if rw == "r":
			num = int(s[2], 0)
			first_byte = 0x80 | addr
			ser.write(b'r' + bytes([first_byte]) + bytes([0] * (num - 1)))
			print_line(ser)
			print_line(ser)
		elif rw == "w":
			assert len(s[2]) == 2
			first_byte = addr
			data = b'w' + bytes([first_byte]) + bytes.fromhex(s[2])
			ser.write(data)
		else:
			print("Invalid command.")

if __name__ == "__main__":
	main()
