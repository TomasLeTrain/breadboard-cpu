from sys import stdin

measurements = []

for line in stdin:
    try:
        # remove newline
        line = line[:-1]
        values = line.split(",")
        curr_measurement = {}
        curr_measurement["clock_cycle"] = values[0]
        curr_measurement["x"] = values[1]
        curr_measurement["y"] = values[2]
        curr_measurement["vsync"] = values[3]
        curr_measurement["hsync"] = values[4]
        curr_measurement["vreset"] = values[5]
        curr_measurement["hreset"] = values[6]
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

print_property("clock_cycle")
print_property("x")
print_property("y")
print_property("vsync")
print_property("hsync")
print_property("vreset")
print_property("hreset")
