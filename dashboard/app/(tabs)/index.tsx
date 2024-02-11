import { Text, View } from "../../components/Themed";
import { APIContext } from "../_layout";
import { gql, useQuery } from "@apollo/client";
import { useContext, useEffect } from "react";
import { StyleSheet } from "react-native";
import {
  CircleMarker,
  MapContainer,
  Popup,
  TileLayer,
  useMap,
} from "react-leaflet";
import L, { LatLngTuple } from "leaflet";
import { GreatCircle } from "arc";
import "leaflet/dist/leaflet.css";
import { useLeafletContext } from "@react-leaflet/core";
// import rgb from "hsv-rgb";
// import sha1 from "sha1";

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
      timestamp
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
      timestamp
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
      timestamp
    }
  }
`;

const ACTIVE_QUERY = gql`
  query {
    activeMinutes
  }
`;

const ACTIVE_QUERY_DAY = gql`
  query ActiveRange($end: String!) {
    activeMinutes(end: $end, duration: 1440)
  }
`;

const COLORS = new Map([
  ["KD9YWS", "blue"],
  ["KD9MEQ", "green"],
]);

function Arc({
  start,
  end,
}: {
  start: { y: number; x: number };
  end: { y: number; x: number };
}) {
  const context = useLeafletContext();

  useEffect(() => {
    const container = context.layerContainer || context.map;

    const generator = new GreatCircle(start, end);
    const line: Array<LatLngTuple> = generator
      .Arc(50, { offset: -1 })
      .geometries[0].coords.map((t: Array<number>) => [t[1], t[0]]);

    var lines = [line];

    const left1 = line.slice(0, -1).map((t) => t[1] < 0 && t[1] > -90);
    const right1 = line.slice(1).map((t) => t[1] >= 0 && t[1] < 90);
    const left2 = line.slice(0, -1).map((t) => t[1] >= 0 && t[1] < 90);
    const right2 = line.slice(1).map((t) => t[1] < 0 && t[1] > -90);
    var idx = left1.findIndex((l, i) => l && right1[i]);
    if (idx < 0) {
      idx = left2.findIndex((l, i) => l && right2[i]);
    }
    idx += 1;

    if (idx) {
      const is_first = left1[idx - 1] && right1[idx - 1];

      const p1 = line[idx - 1];
      const p2 = line[idx];

      const t = p1[1] / (p1[1] - p2[1]);
      const lat = (1 - t) * p1[0] + t * p2[0];

      lines = [line.slice(0, idx), line.slice(idx)];
      lines[0].push([lat, is_first ? 360 : 0]);
      lines[1].unshift([lat, is_first ? 0 : 360]);
    }

    lines = lines.map((l) =>
      l.map((p) => [p[0], p[1] < 0 ? p[1] + 360 : p[1]]),
    );

    lines = lines.flatMap((l) => [
      l,
      l.map((p) => [p[0], p[1] - 360]) as LatLngTuple[],
    ]);

    const arcs = lines.map((l) => L.polyline(l));
    arcs.forEach((arc) => container.addLayer(arc));

    return () => {
      arcs.forEach((arc) => container.removeLayer(arc));
    };
  });

  return null;
}

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
          <ContactMap />
        </View>
        <View style={styles.thin_column}>
          <View style={styles.thin_row}>
            <ActiveMinutes />
          </View>
          <View style={styles.thin_row}>
            <ActiveMinutesLastDay />
          </View>
        </View>
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

function ActiveMinutes() {
  const client = useContext(APIContext);
  if (!client) return null;

  const { loading, error, data } = useQuery(ACTIVE_QUERY, {
    client,
    pollInterval: 5000,
  });

  if (loading) return null;
  if (error) return null;

  const minutes = data.activeMinutes.val.reduce((partial: number, i: number) => partial + i);
  const h = String(Math.floor(minutes / 60));
  const m = String(minutes % 60).padStart(2, '0');

  return (
    <Text>Minutes used: {h}:{m}</Text>
  );
}

function ActiveMinutesLastDay() {
  const client = useContext(APIContext);
  if (!client) return null;

  var now = new Date();
  now.setUTCSeconds(0, 0);
  const end = now.toISOString();

  const { loading, error, data } = useQuery(ACTIVE_QUERY_DAY, {
    client,
    pollInterval: 5000,
    variables: { end },
  });

  console.log(loading, error, data);

  if (loading) return null;
  if (error) return null;

  const minutes = data.activeMinutes.val.reduce((partial: number, i: number) => partial + i);
  const h = String(Math.floor(minutes / 60));
  const m = String(minutes % 60).padStart(2, '0');

  return (
    <Text>Minutes used (last 24h): {h}:{m}</Text>
  );
}

function ContactMap() {
  return (
    <MapContainer
      center={[20, -85]}
      zoom={1}
      scrollWheelZoom={false}
      style={styles.map}
      worldCopyJump={true}
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

  // console.log("Entries:", data.entries);

  if (!data?.entries.length) return null;

  const coords = data.entries.filter((i: any) => i.latitude);
  const most_recent = coords[0];

  // console.log("Last:", last);

  return (
    <>
      {coords.map((i: any) => get_marker(i))}
      <Arc
        start={{ y: 40.42, x: -86.77 }}
        end={{ y: most_recent.latitude, x: most_recent.longitude }}
      />
    </>
  );
}

function get_marker(entry: any) {
  const time_ago = (Date.now() - Date.parse(entry.timestamp)) / 1000;

  const radius = time_ago < 60 ? 5 : time_ago < 600 ? 4 : time_ago < 3600 ? 3 : 2;
  const opacity = time_ago < 120 ? 0.9 : time_ago < 300 ? 0.7 : 0.45;
  // const opacity = Math.max(
  //   Math.min(1 / Math.log10(Math.sqrt(time_ago / 40)), 0.95),
  //   0.25,
  // );

  // const hash = parseInt(sha1(entry.operator).slice(0, 7), 16);
  // const color: number[] = rgb(hash % 100, 99, 99);
  // const color_hex =
  //   "#" + color.map((n) => (n < 16 ? "0" : "") + n.toString(16)).join("");

  const color = COLORS.get(entry.operator) ?? "black";

  return (
    <div key={entry.id}>
      <CircleMarker
        center={[entry.latitude, entry.longitude]}
        radius={radius}
        stroke={false}
        fillOpacity={opacity}
        fillColor={color}
      >
        <Popup>
          {entry.recvCallsign} - {entry.operator}
          <br />
          {entry.freqRx / 1000} - {entry.locationSource}
        </Popup>
      </CircleMarker>
      <CircleMarker
        center={[
          entry.latitude,
          entry.longitude < 0 ? entry.longitude + 360 : entry.longitude - 360,
        ]}
        radius={radius}
        stroke={false}
        fillOpacity={opacity}
        fillColor={color}
      >
        <Popup>
          {entry.recvCallsign} - {entry.operator}
          <br />
          {entry.freqRx / 1000} - {entry.locationSource}
        </Popup>
      </CircleMarker>
    </div>
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
  thin_row: {
    paddingVertical: 5,
  },
  column: {
    flex: 1,
    alignItems: "center",
    justifyContent: "center",
    paddingHorizontal: 10,
    paddingVertical: 2,
  },
  thin_column: {
    flex: 0.3,
    justifyContent: "center",
    paddingHorizontal: 3,
    paddingVertical: 1,
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
