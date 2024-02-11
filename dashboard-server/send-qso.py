import datetime
import readline
import secrets
import socket


UDP_IP = "127.0.0.1"
UDP_PORT = 12063


contact_info = """<?xml version="1.0" encoding="utf-8"?>
<contactinfo>
  <app>N1MM</app>
  <call>{callsign}</call>
  <timestamp>{timestamp}</timestamp>
  <contestname>CWOPS</contestname>
  <contestnr>73</contestnr>
  <mycall>W2XYZ</mycall>
  <band>3.5</band>
  <rxfreq>352519</rxfreq>
  <txfreq>352519</txfreq>
  <operator>TEST</operator>
  <mode>CW</mode>
  <countryprefix>K</countryprefix>
  <wpxprefix>W1</wpxprefix>
  <stationprefix>W2XYZ</stationprefix>
  <continent>NA</continent>
  <snt>599</snt>
  <sntnr>5</sntnr>
  <rcv>599</rcv>
  <rcvnr>0</rcvnr>
  <gridsquare/>
  <exchange1/>
  <section/>
  <comment/>
  <qth/>
  <name/>
  <power/>
  <misctext/>
  <zone>0</zone>
  <prec/>
  <ck>0</ck>
  <ismultiplier1>1</ismultiplier1>
  <ismultiplier2>0</ismultiplier2>
  <ismultiplier3>1</ismultiplier3>
  <points>1</points>
  <radionr>1</radionr>
  <run1run2>1</run1run2>
  <RoverLocation/>
  <RadioInterfaced>1</RadioInterfaced>
  <NetworkedCompNr>0</NetworkedCompNr>
  <IsOriginal>False</IsOriginal>
  <NetBiosName/>
  <IsRunQSO>0</IsRunQSO>
  <StationName>CONTEST-PC</StationName>
  <ID>{id}</ID>
  <IsClaimedQso>1</IsClaimedQso>
</contactinfo>
"""


sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)


def send(callsign):
    t = datetime.datetime.now(tz=datetime.UTC).strftime("%Y-%m-%d %H:%M:%S")
    rand_id = secrets.token_hex(16)
    packet = contact_info.format(timestamp=t, callsign=callsign, id=rand_id)
    print(f"Sending QSO with {callsign} at {t} ({rand_id})")
    sock.sendto(str.encode(packet), (UDP_IP, UDP_PORT))


while True:
    try:
        send(input("Callsign > "))
    except KeyboardInterrupt:
        print("\nBye")
        exit(0)
