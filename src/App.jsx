import React, { useState, useCallback, useEffect } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import debounce from "lodash.debounce";
import "./App.css";
import { ThemeProvider } from "@/components/theme-provider";
import { Navbar } from "@/components/Navbar";
import { ViewPage } from "@/components/ViewPage";
import { SetupPage } from "@/components/SetupPage";

// TODO: Add compacting to Queue

class Queue {
  constructor() {
    this.items = [];
    this.frontIndex = 0;
    this.backIndex = 0;
    this.maxLen = 10;
  }

  enqueue(item) {
    let skip = false;
    function redundancyCheck(value, item) {
      if (value.fileName == item.fileName) {
        skip = true;
      }
    }

    this.items.forEach((value) => redundancyCheck(value, item));

    if (!skip) {
      this.items[this.backIndex] = item;
      this.backIndex++;

      if (this.backIndex > this.maxLen) {
        this.dequeue();
      }
    }
    // return item + ' inserted'
  }

  dequeue() {
    // const item = this.items[this.frontIndex];
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

  setItemAtIndex(item, index) {
    this.items[index] = item;
    if (this.backIndex < index) {
      this.backIndex = index;
    }
  }
}

function App() {
  const [setup, setSetup] = useState(false);
  const [query, setQuery] = useState("");
  const [results, setResults] = useState([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);
  const [recent, setRecent] = useState(new Queue());

  useEffect(() => {
    setupCheck();
    loadRecent();
  }, []);

  const setupCheck = async () => {
    try {
      let flag = await invoke("setup_file_check");
      if(flag) {
        try {
          await invoke("load_setup");
        } catch {
          console.error("Error loading setup file ", error);
        }
      }
      setSetup(flag);
    } catch (error) {
      console.error("Error checking for setup file: ", error);
    }
  };

  const loadRecent = async () => {
    // Invoke the get_recent_data command
    invoke("get_recent_data")
      .then((response) => {
        let recentQueue = new Queue();
        response.map(([key, file]) => {
          recentQueue.setItemAtIndex(
            {
              fileName: file.file_name,
              filePath: file.file_path,
              fileSize: file.file_size,
              fileType: file.file_type,
              creationDate: file.creation_date,
              formattedDate: new Date(
                file.creation_date.secs_since_epoch * 1000
              ).toLocaleString(), // Optional: Format the date
            },
            key
          );
        });
        setRecent(recentQueue);

        // console.log(recentQueue);
      })
      .catch((error) => {
        console.error("Error fetching data:", error);
      });
  };

  const fetchResults = async (query) => {
    setLoading(true);
    setError(null);
    try {
      // Returns an array of objects with filename and details
      const searchResults = await invoke("search_files", { query });
      // Transform the data structure to better work with React
      const formattedResults = searchResults.map(([file_name, details]) => ({
        fileName: file_name,
        filePath: details.file_path,
        fileSize: details.file_size,
        fileType: details.file_type,
        creationDate: details.creation_date,
        formattedDate: new Date(
          details.creation_date.secs_since_epoch * 1000
        ).toLocaleString(), // Optional: Format the date
      }));

      setResults(formattedResults);
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

  const openFile = (file) => {
    if (file["filePath"]) {
      // window.open(filePath, "_blank");
    } else {
      console.warn("No file path provided for opening.");
    }

    updateRecent(file);
  };

  const updateRecent = (file) => {
    let recentQueue = recent;
    // let fileObject = {
    //   file_name: file["filename"],
    //   file_path: file["filePath"],
    //   file_type: file["fileType"],
    //   file_size: file["fileSize"],
    //   creation_date: file["creationDate"],
    // };
    recentQueue.enqueue(file);
    setRecent(recentQueue);

    handleRecent();
  };

  const handleRecent = async () => {
    try {
      let recentQueue = recent;
      let items = recentQueue.getItems();

      items = items.map(
        ({ fileName, filePath, fileSize, fileType, creationDate }) => ({
          file_name: fileName,
          file_path: filePath,
          file_size: fileSize,
          file_type: fileType,
          creation_date: creationDate,
        })
      );

      let itemsMap = Object.assign({}, items);

      await invoke("process_recent", { data: itemsMap });
    } catch (error) {
      console.error("Error: ", error);
      setError("Failed");
    }
  };

  return (
    <ThemeProvider defaultTheme="dark" storageKey="vite-ui-theme">
      {!setup && <SetupPage setup={setup} setupCheck={setupCheck} />}
      {setup && (
        <>
          <Navbar query={query} handleChange={handleChange} />
          <ViewPage
            setup={setup}
            results={results}
            loading={loading}
            error={error}
            query={query}
            openFile={openFile}
            recent={recent}
          ></ViewPage>
        </>
      )}
    </ThemeProvider>
  );
}

export default App;
