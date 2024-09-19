import React, { useState, useCallback, useEffect } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import debounce from "lodash.debounce";
import "./App.css";
import { ThemeProvider } from "@/components/theme-provider";
import { Navbar } from "@/components/Navbar";
import { ViewPage } from "@/components/ViewPage";

class Queue {
  constructor() {
    this.items = {};
    this.frontIndex = 0;
    this.backIndex = 0;
    this.maxLen = 10;
  }
  enqueue(item) {
    this.items[this.backIndex] = item;
    this.backIndex++;

    if (this.backIndex > this.maxLen) {
      this.dequeue();
    }
    // return item + ' inserted'
  }
  dequeue() {
    const item = this.items[this.frontIndex];
    delete this.items[this.frontIndex];
    this.frontIndex++;
    // return item
  }
  getItems() {
    return this.items;
  }
  setItems(items) {
    this.items = items;
  }
}

function App() {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);
  const [recent, setRecent] = useState(new Queue());

  useEffect(() => {
    // Invoke the get_recent_data command
    invoke("get_recent_data")
      .then((response) => {
        console.log(response.recent);
        let recentQueue = new Queue();
        recentQueue.setItems(response.recent);
        setRecent(recentQueue); // response contains the 'recent' array
        console.log("Data received from Rust:", response);
      })
      .catch((error) => {
        console.error("Error fetching data:", error);
      });
  }, []);

  const fetchResults = async (query) => {
    setLoading(true);
    setError(null);
    try {
      const searchResults = await invoke("search_files", { query });
      setResults(searchResults);
    } catch (error) {
      console.error("Error searching files:", error);
      setError("Failed to search files.");
    } finally {
      setLoading(false);
    }
  };

  const debouncedFetchResults = useCallback(
    debounce((query) => fetchResults(query), 300),
    []
  );

  const handleChange = (e) => {
    const newQuery = e.target.value;
    setQuery(newQuery);
    if (newQuery.trim() === "") {
      setResults([]);
    } else {
      debouncedFetchResults(newQuery);
    }
  };

  const openFile = (filePath) => {
    if (filePath) {
      window.open(filePath, "_blank");
    } else {
      console.warn("No file path provided for opening.");
    }

    updateRecent(filePath);
  };

  const updateRecent = (filePath) => {
    let recentQueue = recent;
    recentQueue.enqueue(filePath);
    setRecent(recentQueue);

    handleRecent();
  };

  const handleRecent = async () => {
    try {
      recentQueue = recent;
      items = recentQueue.getItems();
      await invoke("process_recent", { items });
    } catch (error) {
      console.error("Error: ", error);
      setError("Failed");
    }
  };

  return (
    <ThemeProvider defaultTheme="dark" storageKey="vite-ui-theme">
      <div className="App">
        <Navbar query={query} handleChange={handleChange}></Navbar>
        <ViewPage
          results={results}
          loading={loading}
          error={error}
          query={query}
          openFile={openFile}
          recent={recent}
        ></ViewPage>
      </div>
    </ThemeProvider>
  );
}

export default App;
