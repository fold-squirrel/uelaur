# uelaur
**U**el **E**valuation **L**eaf **A**utomation **U**tility in **R**ust \
uelaur for short

## Discription
this is a tool I created for filling the uel paper programmatically,\
it's simply a wrapper around [leafedit](https://github.com/navyleaf/leafedit)
> ### leafedit
> is a command line pdf editor that I wrote for my interest in the
> pdf format as well as my love for CLI and Rust, I'm planning on developing leafedit
> into a tool than can act as the backend for a gui pdf editor

so uelaur reads the data of a csv file plus the co-ordinates of each field
from a toml file then run leafedit with the formated arguments and
leafedit does the heavy lifting

## Usage
first create an empty directory, after that place uelaur in this directory and run it,
if all goes well you should see \
`new project intialized` \
now you will see a directory named **csv_files**
and a file called **uelaur_config.txt** in
the directory you ran it in

now place all the csv files in the the **csv_files** directory \(note all file extentions
are unchecked except for pdf files\)

now configure uelaur by editing **uelaur_config.txt**, you can get the co-ordinates from \
[pdflite.co](https://pdflite.co/simplified-pdf-viewer/index.html) or
[pdfbox](https://pdfbox.apache.org/download.cgi) which is my preferred choice, \
it does require java to be installed but it's much better and easier to use

finally place the uel pdf in the project directory and run it again then \
watch the magic

assuming no error occured, uelaur will displays `done !!!` then exit after
a few seconds and now you will find \
two new directories:

- **uel_papers**: which will contain a directory for every csv file present in **csv_files** \
each directory will have the same name \(minus the extention\) as a csv file

- **review**: which will contain a pdf for each directory in uel\_papers \
the pdf file is the result of merging all the files in the directory inside
**uel_papers** having the same name

it will also contain two files:
- **uel_pdf.patched**: *internal and shouldn't be visible to the user*
- **leafedit(.exe)**: *internal and shouldn't be visible to the user*

but I didn't bother hidding them

## Configuration

configuration file is in the toml format, \
all numeric values must be unsigned integers

### csv related options
- **name_column**: the column number containing each student's name

- **id_column**: the column number containing each student's id

- **mark_column**: the column number containing each student's mark

- **final_mark_column**: the column number containing the final mark

note: columns start from 1, \
also the first row is always skipped as it's usually contains each column's name

### uel pdf related
- **name_postion**: an array containing two  numbers, the first is the x co-ordinate
and the second is the y co-ordinate of the student name postion in the pdf

- **id_postion**: an array containing two  numbers, the first is the x co-ordinate
and the second is the y co-ordinate of the student id postion in the pdf

- **student_field_font_size**: the font size of the student name and id

- **horizontal_marks**: an array of numbers, contains the marks that are listed in the
criteria fields, in other words each mark that is directly above a check mark box

- **horizontal_postions**: an array having the same lenght as the horizontal\_marks array,
each entry is an array containing 2 values, the x and y co-ordinates of where the
check mark should be placed if the student mark is greater than the mark after the
current mark

- **horizontal_feild_font_size**: the font size of the check mark used in the horizontal
mark boxes

- **first_marker_postion**: an array containing two numbers, the first is the x co-ordinate
and the second is the y co-ordinate of the first marker postion in the pdf

- **second_marker_postion**: an array containing two  numbers,
the first is the x co-ordinate and the second is the y co-ordinate of the
second marker postion in the pdf

- **asu_mark_postion**: an array containing two numbers, the first is the x co-ordinate
and the second is the y co-ordinate of the asu mark postion in the pdf

- **uel_mark_postion**: an array containing two numbers, the first is the x co-ordinate
and the second is the y co-ordinate of the uel mark postion in the pdf

- **grade_field_font_size**: the font size used for the marks

- **vertical_marks**: an array containing arrays, each contained array contains to numbers
the first is the lower bound for the uel mark of current range,
the second is the lower bound for the asu mark of the current range, the
current range is the range given to each letter grade,
so F will always have an lower bound of 0 therefor the last entry will always be
[0, 0], also 100 is not an lower bound to any letter grade so the array must never
contain [100, 100]

- **vertical_postions**: an array having the same lenght as the vertical\_marks array,
each entry is an array containing 2 values, the x and y co-ordinates where the
check mark should be placed if the student mark is in the range of the corresponding letter
grade

- **vertical_feild_font_size**: the font size of the check mark used in the vertical
mark boxes

note: all co-ordinates should at the bottom left corner of the field
as the y co-ordinate supplied will act as the baseline for the text

## Error handling
### csv related errors
if the final mark in the second line of the csv file is not present
at the expected column or isn't a valid number (float/int) then the entire
csv file will be skipped, also the final mark is only required in the second line only

if the name or id is empty, or if name contains characters beside
(\[A-Z\]|\[a-z\]|\s|-) *(perl regex)* or if the id contains characters beside ([0-9]|\`|'|\_|,|\\.)
*(perl regex)* then this student will be is skipped

if the student mark is not present or isn't a number (float/int), 0 will be used

if any errors occurred, then a file named **CSV_ERRORS.txt** will be created containing
more information about the errors

### configuration related
the first time an error occurs in the config file then **uelaur_config.txt** will be
renamed to **config_backup.txt** and a new **uelaur_config.txt** will be
created, preserving the valid options and contain more information about each
the errors

if any errors occur for the second time or later then **uelaur_config.txt** will be
overwritten directly with the new error messages

#### config error types
- **missing**: the following keys where not present in the file
- **un-expected**: the following keys are not part of the config
- **misspelled**: the following keys may contain a typo
- repeated: the following keys where found multiple time
- in-correct: the key is valid but the value assigned to is invalid,
refer to the **config_backup.txt** as it might help

### Other
all other errors are printed to standard out

## Limitations
windows cmd has a feature called QuickEdit which will block all print operations if
the user selects text, it will unblock them once any key is pressed and I couldn't
find a way to disable this in code so if a user clicked on
the console they need to press
any key for messages to be displayed again, \
note that all print operations are called from a separate thread
than the main thread so uelaur
could exit without displaying error messages and warnings


also I don't know how to sign binarys so if you do download uelaur from the here,
your browser will block it
or warn you that it's not a commonly download executable, after that if you try to run
it windows will block it with a blue screen because it contains a "mark of the web" and
you will need to right click the executable and click properties and check the
unblock box


the macos version is untested and will most likely fail as the leafedit
bundled with uelaur is unsigned, because I don't own a mac or an apple developer account \
so if your can compile, sign and test it, then open a pull request with your compiled binary

## final word
I'd thank kevin Chrief for reviewing with the README

I don't plan on maintaining this project so that's why the variables and function
names aren't discriptive, and the order of the function is random \
but if someone is willing to maintain uelaur, I will consider rewriting it in a cleaner way

\
\
\
\
also I'd like a few bonus marks as compensation for my I hard work \
*I really need the extra marks*
