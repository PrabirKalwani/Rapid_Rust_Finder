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
  const [start, setStart] = useState(false);
  const [query, setQuery] = useState("");
  const [results, setResults] = useState([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);
  const [recent, setRecent] = useState(new Queue());
  const [selectedFile, setSelectedFile] = useState(null);

  useEffect(() => {
    setupCheck().then((flag) => {
      if (flag) {
        startup();
        loadRecent();
      }
    });
  }, []);

  const startup = async () => {
    try {
      await invoke("startup");
      setStart(true);
    } catch (error) {
      console.error("Error starting up: ", error);
      setStart(false);
    }
  };

  const setupCheck = async () => {
    try {
      let flag = await invoke("setup_file_check");
      if (flag) {
        try {
          await invoke("load_setup");
        } catch {
          console.error("Error loading setup file ", error);
        }
      }
      setSetup(flag);
      return flag;
    } catch (error) {
      console.error("Error checking for setup file: ", error);
    }
  };

  const loadRecent = async () => {
    // Invoke the get_recent_data command
    invoke("get_recent_data")
      .then((response) => {
        let recentQueue = new Queue();
        console.log(response);
        response.map(([key, file]) => {
          recentQueue.setItemAtIndex(
            {
              fileName: file[0],
              filePath: file[1],
            },
            key
          );
        });
        setRecent(recentQueue);
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
      const formattedResults = searchResults.map(([file_name, file_path]) => ({
        fileName: file_name,
        filePath: file_path,
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

      invoke("open_file", { path: file.filePath })
        .then(() => console.log(`File opened: ${file.filePath}`))
        .catch((error) => console.error("Failed to open file:", error));
    } else {
      console.warn("No file path provided for opening.");
    }

    updateRecent(file);
  };

  const updateRecent = (file) => {
    let recentQueue = recent;
    recentQueue.enqueue(file);
    setRecent(recentQueue);

    handleRecent();
  };

  const handleRecent = async () => {
    try {
      let recentQueue = recent;
      let items = recentQueue.getItems();

      let itemsArray = Object.entries(items).map(([key, value]) => [
        parseInt(key), // Convert key to integer
        [value.fileName, value.filePath], // Value as a tuple of [fileName, filePath]
      ]);

      await invoke("process_recent", { data: itemsArray });
    } catch (error) {
      console.error("Error: ", error);
      setError("Failed");
    }
  };

  const getFileIcon = (filename) => {
    const split = filename.split(".");
    if(split.length < 2){
      return "latte/_folder.svg"
    }
    else {
      const extension = split.pop().toLowerCase();
      switch (extension) {
        case "pdf":
          return "latte/pdf.svg";
        case "jpg":
        case "jpeg":
        case "png":
          return "latte/image.svg";
        case "csv":
          return "latte/csv.svg";
        case "xls":
        case "xlsx":
          return "latte/ms-excel.svg";
        case "doc":
        case "docx":
          return "latte/ms-word.svg";
        case "pptx":
          return "latte/ms-powerpoint.svg";
        case "txt":
          return "latte/text.svg";
  
        case "js":
          return "latte/javascript.svg";
        case "ts":
          return "latte/typescript.svg";
        case "css":
          return "latte/css.svg";
        case "html":
          return "latte/html.svg";
        case "py":
          return "latte/python.svg";
        case "rs":
          return "latte/rust.svg";
  
        case "exe":
          return "latte/exe.svg";
        case "":
          return "latte/_folder.svg"
        default:
          return "latte/_file.svg";
      }
    }
  };

  return (
    <ThemeProvider defaultTheme="dark" storageKey="vite-ui-theme">
      {(!setup || !start) && (
        <SetupPage
          startup={startup}
          setupCheck={setupCheck}
          loadRecent={loadRecent}
        />
      )}
      {setup && start && (
        <>
          <ViewPage
            setup={setup}
            results={results}
            loading={loading}
            error={error}
            query={query}
            openFile={openFile}
            recent={recent}
            getFileIcon={getFileIcon}
            handleChange={handleChange}
            selectedFile={selectedFile}
            setSelectedFile={setSelectedFile}
          ></ViewPage>
        </>
      )}
    </ThemeProvider>
  );
}

export default App;
