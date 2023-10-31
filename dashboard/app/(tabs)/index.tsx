import { Text, View } from "../../components/Themed";
import { APIContext } from "../_layout";
import { gql, useQuery } from "@apollo/client";
import { useContext, useEffect } from "react";
import { StyleSheet } from "react-native";
import { MapContainer, Marker, Popup, TileLayer, useMap } from "react-leaflet";
import L from "leaflet";
import "leaflet/dist/leaflet.css";

const LATEST_QUERY = gql`
  query {
    latest {
      recvCallsign
      freqRx
      mode
      operator
      locationSource
      latitude
      longitude
    }
  }
`;

const LATEST_SUBSCRIPTION = gql`
  subscription {
    latest {
      recvCallsign
      freqRx
      mode
      operator
      locationSource
      latitude
      longitude
    }
  }
`;

const LOCATION_QUERY = gql`
  query {
    entries {
      id
      recvCallsign
      freqRx
      mode
      operator
      locationSource
      latitude
      longitude
    }
  }
`;

const dot_icon = L.icon({ iconUrl: "/assets/images/circle-icon.png", iconSize: [4, 4] });

export default function TabOneScreen() {
  // <View
  //   style={styles.separator}
  //   lightColor="#eee"
  //   darkColor="rgba(255,255,255,0.1)"
  // />
  return (
    <View style={styles.container}>
      <View style={styles.row}>
        <View style={styles.container}>
          <LatestContact />
          <Map />
        </View>
        <Text>Column 2</Text>
      </View>
    </View>
  );
}

function LatestContact() {
  const client = useContext(APIContext);
  if (!client) return `Loading`;

  const { loading, error, data, subscribeToMore } = useQuery(LATEST_QUERY, {
    client,
  });

  useEffect(() =>
    subscribeToMore({
      document: LATEST_SUBSCRIPTION,
      updateQuery: (prev, { subscriptionData }) => {
        if (!subscriptionData?.data?.latest) return prev;
        return subscriptionData.data;
      },
    }),
  );

  if (loading) return `Loading`;
  if (error) return `Error! ${error}`;

  return (
    <View style={styles.row}>
      <View style={styles.column}>
        <Text style={styles.center_title}>Last QSO</Text>
        <Text style={styles.center}>{data.latest?.recvCallsign}</Text>
      </View>
      <View style={styles.column}>
        <Text style={styles.center_title}>Frequency</Text>
        <Text style={styles.center}>{data.latest?.freqRx / 1000}</Text>
      </View>
      <View style={styles.column}>
        <Text style={styles.center_title}>Mode</Text>
        <Text style={styles.center}>{data.latest?.mode}</Text>
      </View>
      <View style={styles.column}>
        <Text style={styles.center_title}>Operator</Text>
        <Text style={styles.center}>{data.latest?.operator}</Text>
      </View>
      <View style={styles.column}>
        <Text style={styles.center_title}>Location Source</Text>
        <Text style={styles.center}>{data.latest?.locationSource}</Text>
      </View>
    </View>
  );
}

function Map() {
  return (
    <MapContainer
      center={[20, 0]}
      zoom={1}
      scrollWheelZoom={false}
      style={styles.map}
    >
      <TileLayer
        attribution='&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors'
        url="https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png"
      />
      <MapMarkers />
    </MapContainer>
  );
}

function MapMarkers() {
  const map = useMap();

  const client = useContext(APIContext);
  if (!client) return null;

  const { loading, error, data } = useQuery(LOCATION_QUERY, {
    client,
    pollInterval: 2000,
  });

  if (loading) return null;
  if (error) return null;

  console.log(data.entries);

  return (
    <div>
      {data.entries.filter((i: any) => i.latitude).filter((i: any) => i.operator == "KD9YWS").map((i: any) => get_marker(i))}
    </div>
  );
}

function get_marker(entry: any) {
  console.log(entry, dot_icon);
  return (
    <Marker position={[entry.latitude, entry.longitude]} icon={dot_icon} key={entry.id}>
      <Popup>
        {entry.recvCallsign} - {entry.operator} - {entry.freqRx / 1000} - {entry.locationSource}
      </Popup>
    </Marker>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    alignItems: "center",
    justifyContent: "center",
  },
  row: {
    width: "100%",
    flex: 1,
    flexDirection: "row",
  },
  column: {
    flex: 1,
    alignItems: "center",
    justifyContent: "center",
    paddingHorizontal: 10,
    paddingVertical: 2,
  },
  center_title: {
    fontWeight: "bold",
    textAlign: "center",
  },
  center: {
    textAlign: "center",
  },
  map: {
    height: "80%",
    width: "90%",
    marginBottom: 15,
  },
  separator: {
    marginVertical: 30,
    height: 1,
    width: "80%",
  },
});
