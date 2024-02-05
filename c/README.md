# FFI interface

FFI (Foreign Function Interface) is a mechanism that allows programs written in
one programming language to use functions or libraries written in another
language.

This crate provides financial functions via FFI, so they can be used with other
languages, such as C, C#, Ruby, NodeJS and many others.

## Building

```bash
cargo build --release
```

## `C`

```c
#include <stdio.h>
#include <stdlib.h>

// Function to calculate XIRR in C
extern int xirr(
    long* dates,
    int date_count,
    double* amounts,
    int amount_count,
    double guess,
    unsigned short day_count,
    double* result
);

int main() {
    // Input data
    long timestamp[] = {1577833200, 1580511600, 1583017200};
    double amounts[] = {-1000, 750, 500};

    // Call the xirr function
    double rate;
    int result_code = xirr(timestamp, 3, amounts, 3, 0.1, 0, &rate);

    // Check the result code and process the result
    if (result_code == 0) {
        printf("XIRR result: %lf\n", rate);
    } else {
        printf("Error calculating XIRR. Result code: %d\n", result_code);
    }

    return 0;
}
```

Compilation:

```bash
gcc -L ./target/release -l xirr example.c
LD_LIBRARY_PATH=./target/release ./a.out
```

## `C#`

```csharp
using System;
using System.Runtime.InteropServices;

public class FFIExample
{
    // Change the library name accordingly
    const string __DllPath = "./target/release/libxirr.so";

    // Import the foreign functions
    [DllImport(__DllPath)]
    private static extern int xirr(
        long[] dates,
        int dates_length,
        double[] values,
        int values_length,
        double guess,
        ushort day_count,
        out double result
    );

    private static DateTime UNIX_EPOCH = new DateTime(1970, 1, 1);

    public static double XIRR(DateTime[] dates, double[] values, double? guess)
    {
        // Prepare data for the foreign function
        long[] timestamps = dates.Select(DateTimeToUnixTimestamp).ToArray();

        // Call the foreign function
        int resultCode = xirr(
            timestamps,
            timestamps.Length,
            values,
            values.Length,
            guess ?? 0.1,
            0,
            out double result
        );

        // Check the result code and process the result as needed
        if (resultCode != 0)
        {
            throw new Exception($"Error Code: {resultCode}");
        }

        return result;
    }

    private static long DateTimeToUnixTimestamp(DateTime datetime)
    {
        return (long)(datetime.Subtract(UNIX_EPOCH).TotalSeconds);
    }

    public static void Main()
    {
        // Example data
        var values = new double[]
        {
            -1000.0,
            750.0,
            500.0,
        };

        var dates = new DateTime[]
        {
            new DateTime(2020, 1, 1),
            new DateTime(2020, 2, 1),
            new DateTime(2020, 3, 1),
        };

        // Call foreign function
        double r1 = XIRR(dates, values, null);
        Console.WriteLine($"XIRR: {r1}");
    }
}
```

Compilation:

```bash
dotnet run
```
