import { Button } from "@/components/ui/button";
import {
  Table,
  TableBody,
  TableCaption,
  TableCell,
  TableFooter,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";

export const ResultsContainer = ({
  className,
  results,
  loading,
  error,
  query,
  openFile,
}) => {
  className += " col-span-7 p-2";

  const getFileIcon = (filename) => {
    const extension = filename.split(".").pop().toLowerCase();
    switch (extension) {
      case "pdf":
        return "pdf.png";
      case "docx":
        return "docx.png";
      case "xlsx":
        return "xlsx.png";
      case "jpg":
      case "jpeg":
        return "image.png";
      case "png":
        return "image.png";
      default:
        return "default.png";
    }
  };

  return (
    <section className={className}>
      <div>
        <span className="text-2xl">Results</span>
      </div>
      {results.length === 0 && !loading && !error && query.trim() !== "" && (
        <p className="text-muted-foreground">No results found</p>
      )}
      {results.length > 0 && (
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead className="">File Name</TableHead>
              <TableHead>File Type</TableHead>
              <TableHead>File Size</TableHead>
              <TableHead className="">Creation Date</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {results.map((file, index) => (
              <TableRow key={index} onClick={() => openFile(file)}>
                <TableCell className="font-medium">
                  {file["fileName"]}
                </TableCell>
                <TableCell>{file["fileType"]}</TableCell>
                <TableCell>{file["fileSize"]} B</TableCell>
                <TableCell className="">{file["formattedDate"]}</TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      )}
    </section>
  );
};
