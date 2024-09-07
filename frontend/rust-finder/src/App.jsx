import React, { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import debounce from 'lodash.debounce';
import './App.css'; 

function App() {
  const [query, setQuery] = useState('');
  const [results, setResults] = useState([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);

  
  const fetchResults = async (query) => {
    setLoading(true);
    setError(null);
    try {
      const searchResults = await invoke('search_files', { query });
      setResults(searchResults);
    } catch (error) {
      console.error('Error searching files:', error);
      setError('Failed to search files.');
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
    if (newQuery.trim() === '') {
      setResults([]);
    } else {
      debouncedFetchResults(newQuery);
    }
  };

  const openFile = (filePath) => {
    if (filePath) {
      window.open(filePath, '_blank');
    } else {
      console.warn('No file path provided for opening.');
    }
  };

  return (
    <div className="App">
      <header className="App-header">
        <h1>File Explorer</h1>
        <div className="search-container">
          <input
            type="text"
            value={query}
            onChange={handleChange}
            placeholder="Search files..."
            className="search-input"
          />
        </div>
      </header>
      <main>
        <div className="results-container">
          {query.trim() === '' && !loading && !error && (
            <p className="placeholder-message">Type something here...</p>
          )}
          {loading && <p className="loading">Loading...</p>}
          {error && <p className="error">{error}</p>}
          {results.length === 0 && !loading && !error && query.trim() !== '' && (
            <p className="no-results">No results found</p>
          )}
          {results.length > 0 && (
            <ul className="results-list">
              {results.map(([highlightedFilename, filePath], index) => (
                <li
                  key={index}
                  className="result-item"
                  onClick={() => openFile(filePath)}
                >
                  <img className="icon" src={`/icons/${getFileIcon(highlightedFilename)}`} alt="file-icon" />
                  <span className="filename" dangerouslySetInnerHTML={{ __html: highlightedFilename }}></span>
                  <span className="file-path">{filePath}</span>
                </li>
              ))}
            </ul>
          )}
        </div>
      </main>
    </div>
  );
}


const getFileIcon = (filename) => {
  const extension = filename.split('.').pop().toLowerCase();
  switch (extension) {
    case 'pdf':
      return 'pdf.png';
    case 'docx':
      return 'docx.png';
    case 'xlsx':
      return 'xlsx.png';
    case 'jpg':
    case 'jpeg':
      return 'image.png';
    case 'png':
      return 'image.png';
    default:
      return 'default.png'; 
  }
};

export default App;
