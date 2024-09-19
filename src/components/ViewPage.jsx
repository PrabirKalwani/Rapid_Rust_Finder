import { Sidebar } from "@/components/Sidebar";
import { ResultsContainer } from "@/components/ResultsContainer";
import { RecentsContainer } from "@/components/RecentsContainer";

export const ViewPage = ({
  results,
  loading,
  error,
  query,
  openFile,
  recent,
}) => {
  return (
    <main className="grid grid-cols-9">
      <Sidebar />
      {query != "" && (
        <ResultsContainer
          results={results}
          loading={loading}
          error={error}
          query={query}
          openFile={openFile}
        />
      )}
      {query == "" && <RecentsContainer recent={recent} />}
    </main>
  );
};
