from sys import stdin

measurements = []

range_measurements = (210,3000)

for line in stdin:
    try:
        # remove newline
        line = line[:-1]
        values = line.split(",")
        curr_measurement = {}
        curr_measurement["c_{clockCycle}"] = values[0]
        curr_measurement["x_{1}"] = values[1]
        curr_measurement["y_{1}"] = values[2]
        curr_measurement["v_{vsync}"] = values[3]
        curr_measurement["h_{hsync}"] = values[4]
        curr_measurement["v_{vreset}"] = values[5]
        curr_measurement["h_{hreset}"] = values[6]
        measurements.append(curr_measurement)
    except:
        pass
        # print(line)
        # exit()


def print_property(name):
    print(f"{name} = \\left[",end="");
    for i, measurement in enumerate(measurements):
        if( i != len(measurements)-1):
            print(f"{measurement[name]},",end="")
        else:
            print(f"{measurement[name]}\\right]")

measurements = measurements[range_measurements[0]:range_measurements[1]]

print_property("c_{clockCycle}")
print_property("x_{1}")
print_property("y_{1}")
print_property("v_{vsync}")
print_property("h_{hsync}")
print_property("v_{vreset}")
print_property("h_{hreset}")
print("total number of measurements:",len(measurements))
